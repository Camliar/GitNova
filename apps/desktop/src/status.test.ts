import { beforeEach, describe, expect, it, vi } from "vitest";
import { getWorkingTreeStatus } from "./status";

const mocks = vi.hoisted(() => ({ requestCore: vi.fn() }));
vi.mock("./core", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./core")>()),
  requestCore: mocks.requestCore,
}));

describe("working tree status boundary", () => {
  beforeEach(() => vi.clearAllMocks());

  it("requests a snapshot without repository parameters", async () => {
    const snapshot = {
      branch: { head: "main", oid: "a".repeat(40), upstream: null, ahead: 0, behind: 0 },
      entries: [],
    };
    mocks.requestCore.mockResolvedValue({ jsonrpc: "2.0", id: 3, result: snapshot });

    await expect(getWorkingTreeStatus()).resolves.toEqual(snapshot);
    expect(mocks.requestCore).toHaveBeenCalledWith("repository/status", null);
  });

  it("maps stable errors without forwarding Core details", async () => {
    mocks.requestCore.mockResolvedValue({
      jsonrpc: "2.0",
      id: 3,
      error: {
        code: -32100,
        message: "System Git could not read status",
        data: { stableCode: "git.command_failed", retryable: true, details: { stderr: "secret" } },
      },
    });

    await expect(getWorkingTreeStatus()).rejects.toEqual({
      code: "git.command_failed",
      message: "System Git could not read status",
      retryable: true,
    });
  });
});
