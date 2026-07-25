const { spawn } = require("node:child_process");
const { PROTOCOL_VERSION } = require("./protocolVersion");

const MAX_FRAME_BYTES = 16 * 1024 * 1024;
const REQUEST_TIMEOUT_MS = 15_000;

class FrameDecoder {
  constructor(onMessage) {
    this.onMessage = onMessage;
    this.buffer = Buffer.alloc(0);
  }

  push(chunk) {
    this.buffer = Buffer.concat([this.buffer, chunk]);
    for (;;) {
      const marker = this.buffer.indexOf("\r\n\r\n");
      if (marker < 0) return;
      const header = this.buffer.subarray(0, marker).toString("ascii");
      const fields = header.split("\r\n").filter(Boolean);
      const lengths = fields.filter((line) => /^content-length:/i.test(line));
      if (lengths.length !== 1) throw new Error("Invalid Core frame header");
      const length = Number(lengths[0].split(":", 2)[1].trim());
      if (!Number.isSafeInteger(length) || length < 0 || length > MAX_FRAME_BYTES) throw new Error("Invalid Core frame length");
      const end = marker + 4 + length;
      if (this.buffer.length < end) return;
      const body = this.buffer.subarray(marker + 4, end);
      this.buffer = this.buffer.subarray(end);
      this.onMessage(JSON.parse(body.toString("utf8")));
    }
  }
}

function frameMessage(value) {
  const body = Buffer.from(JSON.stringify(value));
  if (body.length > MAX_FRAME_BYTES) throw new Error("Core request is too large");
  return Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
}

class CoreClient {
  constructor(program) {
    this.program = program;
    this.child = undefined;
    this.nextId = 1;
    this.pending = new Map();
  }

  get running() {
    return Boolean(this.child);
  }

  async start() {
    if (this.child) return;
    const child = spawn(this.program, [], { stdio: ["pipe", "pipe", "pipe"], shell: false, windowsHide: true });
    this.child = child;
    child.stderr.on("data", () => {});
    const decoder = new FrameDecoder((message) => this.receive(message));
    child.stdout.on("data", (chunk) => {
      try { decoder.push(chunk); } catch { this.abort("Core returned an invalid protocol frame"); }
    });
    child.once("error", () => this.failAll("Core could not be started"));
    child.once("exit", () => this.failAll("Core exited"));
    const initialized = await this.request("gitnova/initialize", {
      clientInfo: { name: "gitnova-vscode", version: "0.1.0" },
      protocolVersion: PROTOCOL_VERSION,
      capabilities: { cancellation: true },
    });
    if (String(initialized.protocolVersion).split(".")[0] !== PROTOCOL_VERSION.split(".")[0]) {
      await this.dispose();
      throw new Error("Core protocol is incompatible");
    }
  }

  request(method, params) {
    if (!this.child?.stdin.writable) return Promise.reject(new Error("Core is not running"));
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error("Core request timed out"));
      }, REQUEST_TIMEOUT_MS);
      this.pending.set(id, { resolve, reject, timer });
      this.child.stdin.write(frameMessage({ jsonrpc: "2.0", id, method, params }));
    });
  }

  receive(message) {
    if (message?.jsonrpc !== "2.0" || !this.pending.has(message.id)) return;
    const pending = this.pending.get(message.id);
    this.pending.delete(message.id);
    clearTimeout(pending.timer);
    if (message.error) pending.reject(new Error(message.error.message || "Core request failed"));
    else if (Object.prototype.hasOwnProperty.call(message, "result")) pending.resolve(message.result);
    else pending.reject(new Error("Core returned an invalid response"));
  }

  failAll(message) {
    for (const pending of this.pending.values()) {
      clearTimeout(pending.timer);
      pending.reject(new Error(message));
    }
    this.pending.clear();
    this.child = undefined;
  }

  abort(message) {
    const child = this.child;
    this.failAll(message);
    child?.kill();
  }

  async dispose() {
    const child = this.child;
    if (!child) return;
    try {
      await this.request("gitnova/shutdown", null);
      child.stdin.write(frameMessage({ jsonrpc: "2.0", method: "exit", params: null }));
    } catch {}
    setTimeout(() => child.kill(), 2_000).unref();
    this.child = undefined;
  }
}

module.exports = { CoreClient, FrameDecoder, frameMessage, MAX_FRAME_BYTES };
