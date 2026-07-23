import { beforeEach, describe, expect, it, vi } from "vitest";
import { getFileDiff } from "./diff";

const mocks = vi.hoisted(() => ({ requestCore: vi.fn() }));
vi.mock("./core", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./core")>()),
  requestCore: mocks.requestCore,
}));

describe("structured file diff boundary", () => {
  beforeEach(() => vi.clearAllMocks());

  it("passes the status path, selected scope, and bounded context to Core", async () => {
    const result = { oldPath: "a.ts", newPath: "a.ts", isBinary: false, hunks: [] };
    mocks.requestCore.mockResolvedValue({ jsonrpc: "2.0", id: 4, result });

    await expect(getFileDiff("a.ts", "staged")).resolves.toEqual(result);
    expect(mocks.requestCore).toHaveBeenCalledWith("repository/diff", {
      path: "a.ts",
      scope: "staged",
      contextLines: 3,
    });
  });

  it("returns a stable error without Core details", async () => {
    mocks.requestCore.mockResolvedValue({
      jsonrpc: "2.0",
      id: 4,
      error: {
        code: -32101,
        message: "Diff could not be read",
        data: { stableCode: "git.diff_parse_failed", retryable: false, details: { patch: "secret" } },
      },
    });

    await expect(getFileDiff("a.ts", "workingTree")).rejects.toEqual({
      code: "git.diff_parse_failed",
      message: "Diff could not be read",
      retryable: false,
    });
  });
});
