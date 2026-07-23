import { beforeEach, describe, expect, it, vi } from "vitest";
import { getCommitGraph, HISTORY_PAGE_SIZE } from "./history";

const mocks = vi.hoisted(() => ({ requestCore: vi.fn() }));
vi.mock("./core", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./core")>()),
  requestCore: mocks.requestCore,
}));

describe("commit graph boundary", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.requestCore.mockResolvedValue({ jsonrpc: "2.0", id: 5, result: { nodes: [], nextCursor: null } });
  });

  it("requests a bounded first page", async () => {
    await getCommitGraph();
    expect(mocks.requestCore).toHaveBeenCalledWith("repository/graph", { limit: HISTORY_PAGE_SIZE });
  });

  it("returns the opaque Core cursor unchanged", async () => {
    await getCommitGraph("opaque:do-not-parse");
    expect(mocks.requestCore).toHaveBeenCalledWith("repository/graph", {
      limit: HISTORY_PAGE_SIZE,
      cursor: "opaque:do-not-parse",
    });
  });

  it("maps stable errors without provider or Git details", async () => {
    mocks.requestCore.mockResolvedValue({
      jsonrpc: "2.0",
      id: 5,
      error: { code: -32111, message: "History cursor is invalid", data: { stableCode: "history.invalid_cursor", retryable: false, details: { oid: "secret" } } },
    });
    await expect(getCommitGraph("opaque")).rejects.toEqual({
      code: "history.invalid_cursor",
      message: "History cursor is invalid",
      retryable: false,
    });
  });
});
