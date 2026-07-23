import { beforeEach, describe, expect, it, vi } from "vitest";
import { openRepository, selectRepositoryDirectory } from "./repository";

const mocks = vi.hoisted(() => ({ dialogOpen: vi.fn(), requestCore: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: mocks.dialogOpen }));
vi.mock("./core", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./core")>()),
  requestCore: mocks.requestCore,
}));

describe("repository boundary", () => {
  beforeEach(() => vi.clearAllMocks());

  it("requests one directory and passes its opaque path to Core", async () => {
    mocks.dialogOpen.mockResolvedValue("/repo");
    mocks.requestCore.mockResolvedValue({ jsonrpc: "2.0", id: 2, result: { kind: "bare" } });

    expect(await selectRepositoryDirectory()).toBe("/repo");
    await openRepository("/repo");
    expect(mocks.dialogOpen).toHaveBeenCalledWith({ directory: true, multiple: false });
    expect(mocks.requestCore).toHaveBeenCalledWith("repository/open", { path: "/repo" });
  });

  it("maps a Core domain error without exposing details", async () => {
    mocks.requestCore.mockResolvedValue({
      jsonrpc: "2.0",
      id: 2,
      error: {
        code: -32020,
        message: "Repository was not found",
        data: { stableCode: "repository.not_found", retryable: true, details: { path: "/secret" } },
      },
    });

    await expect(openRepository("/secret")).rejects.toEqual({
      code: "repository.not_found",
      message: "Repository was not found",
      retryable: true,
    });
  });
});
