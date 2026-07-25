const test = require("node:test");
const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { resolve } = require("node:path");
const { FrameDecoder, frameMessage, MAX_FRAME_BYTES } = require("./coreClient");
const { PROTOCOL_VERSION } = require("./protocolVersion");
const { escapeHtml, renderTrace } = require("./render");

test("decodes split and adjacent protocol frames", () => {
  const messages = [];
  const decoder = new FrameDecoder((message) => messages.push(message));
  const first = frameMessage({ jsonrpc: "2.0", id: 1, result: { ok: true } });
  const second = frameMessage({ jsonrpc: "2.0", id: 2, result: null });
  decoder.push(first.subarray(0, 9));
  decoder.push(Buffer.concat([first.subarray(9), second]));
  assert.deepEqual(messages.map((message) => message.id), [1, 2]);
});

test("rejects duplicate and oversized frame lengths", () => {
  const decoder = new FrameDecoder(() => {});
  assert.throws(() => decoder.push(Buffer.from("Content-Length: 1\r\nContent-Length: 1\r\n\r\nx")));
  assert.throws(() => decoder.push(Buffer.from(`Content-Length: ${MAX_FRAME_BYTES + 1}\r\n\r\n`)));
});

test("runtime protocol version stays aligned with the schema", () => {
  const schema = JSON.parse(readFileSync(resolve(__dirname, "../../../sdk/protocol/gitnova-protocol.schema.json"), "utf8"));
  assert.equal(PROTOCOL_VERSION, schema.properties.protocolVersion.const);
});

test("webview escapes repository content and disables scripts", () => {
  assert.equal(escapeHtml("<script>'x'</script>"), "&lt;script&gt;&#39;x&#39;&lt;/script&gt;");
  const html = renderTrace({ pullRequest: { number: 1, title: "<img>", nameWithOwner: "o/r", state: "merged", commits: [] }, relationship: { classification: "squashCandidate", confidence: "medium", mergeCommitOid: null } });
  assert.ok(html.includes("default-src 'none'"));
  assert.ok(html.includes("&lt;img&gt;"));
  assert.ok(!html.includes("<script"));
});
