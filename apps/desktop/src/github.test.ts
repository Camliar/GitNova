import { beforeEach, describe, expect, it, vi } from "vitest";
import { getGitHubPullRequest, getGitHubRepository } from "./github";

const mocks = vi.hoisted(() => ({ requestCore: vi.fn() }));
vi.mock("./core", async (importOriginal) => ({ ...(await importOriginal<typeof import("./core")>()), requestCore: mocks.requestCore }));

describe("GitHub provider boundary", () => {
  beforeEach(() => { vi.clearAllMocks(); mocks.requestCore.mockResolvedValue({ jsonrpc: "2.0", id: 7, result: {} }); });
  it("resolves repository only through an explicit empty-parameter request", async () => {
    await getGitHubRepository();
    expect(mocks.requestCore).toHaveBeenCalledWith("github/repository", {});
  });
  it("binds a PR number to the normalized repository identity", async () => {
    await getGitHubPullRequest(42, "owner/repo");
    expect(mocks.requestCore).toHaveBeenCalledWith("github/pullRequest", { number: 42, nameWithOwner: "owner/repo" });
  });
  it("maps stable auth errors without credential details", async () => {
    mocks.requestCore.mockResolvedValue({ jsonrpc: "2.0", id: 7, error: { code: -32124, message: "GitHub authentication is required", data: { stableCode: "github.auth_required", retryable: true, details: { token: "secret" } } } });
    await expect(getGitHubRepository()).rejects.toEqual({ code: "github.auth_required", message: "GitHub authentication is required", retryable: true });
  });
});
