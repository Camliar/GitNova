import { beforeEach, describe, expect, it, vi } from "vitest";
import { commitStaged, createLocalBranch, getRepositoryReferences, switchLocalBranch } from "./mutations";

const mocks = vi.hoisted(() => ({ requestCore: vi.fn(), coreResult: vi.fn((response) => response.result) }));
vi.mock("./core", () => mocks);

describe("Desktop mutation boundary", () => {
  beforeEach(() => { vi.clearAllMocks(); mocks.requestCore.mockResolvedValue({ result: {} }); });
  it("sends only typed explicit parameters to Core", async () => {
    await getRepositoryReferences(); await commitStaged("message"); await createLocalBranch("topic"); await switchLocalBranch("main");
    expect(mocks.requestCore).toHaveBeenNthCalledWith(1, "repository/references", null);
    expect(mocks.requestCore).toHaveBeenNthCalledWith(2, "repository/commit", { message: "message" });
    expect(mocks.requestCore).toHaveBeenNthCalledWith(3, "repository/createBranch", { name: "topic" });
    expect(mocks.requestCore).toHaveBeenNthCalledWith(4, "repository/switchBranch", { name: "main" });
  });
});
