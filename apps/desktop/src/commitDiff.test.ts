import { beforeEach, describe, expect, it, vi } from "vitest";
import { getCommitDiff } from "./commitDiff";

const mocks = vi.hoisted(() => ({ requestCore: vi.fn() }));
vi.mock("./core", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./core")>()),
  requestCore: mocks.requestCore,
}));

describe("structured commit diff boundary", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.requestCore.mockResolvedValue({ jsonrpc: "2.0", id: 6, result: { commit: {}, parentOid: null, files: [] } });
  });

  it("omits parentOid for root and single-parent automatic selection", async () => {
    await getCommitDiff("a".repeat(40));
    expect(mocks.requestCore).toHaveBeenCalledWith("repository/commitDiff", {
      oid: "a".repeat(40),
      contextLines: 3,
    });
  });

  it("passes an explicitly selected merge parent unchanged", async () => {
    await getCommitDiff("a".repeat(40), "b".repeat(40));
    expect(mocks.requestCore).toHaveBeenCalledWith("repository/commitDiff", {
      oid: "a".repeat(40),
      parentOid: "b".repeat(40),
      contextLines: 3,
    });
  });

  it("maps stable errors without raw object details", async () => {
    mocks.requestCore.mockResolvedValue({
      jsonrpc: "2.0",
      id: 6,
      error: { code: -32115, message: "Choose a merge parent", data: { stableCode: "commit.parent_required", retryable: false, details: { object: "secret" } } },
    });
    await expect(getCommitDiff("a".repeat(40))).rejects.toEqual({
      code: "commit.parent_required",
      message: "Choose a merge parent",
      retryable: false,
    });
  });
});
