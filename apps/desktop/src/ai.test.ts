import { beforeEach, describe, expect, it, vi } from "vitest";
import { generateAiCommitDraft, previewAiInput } from "./ai";

const mocks = vi.hoisted(() => ({ requestCore: vi.fn(), coreResult: vi.fn((response) => response.result) }));
vi.mock("./core", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./core")>()),
  requestCore: mocks.requestCore,
  coreResult: mocks.coreResult,
}));

describe("Desktop AI Core client", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.requestCore.mockResolvedValue({ result: {} });
  });

  it("sends only provider configuration, exclusions and preview-bound confirmation", async () => {
    const provider = { kind: "openAi" as const, model: "user-model" };
    await previewAiInput(provider, ["private.json"]);
    await generateAiCommitDraft("preview-1", provider, ["private.json"], true);

    expect(mocks.requestCore).toHaveBeenNthCalledWith(1, "ai/inputPreview", {
      provider,
      excludedPaths: ["private.json"],
    });
    expect(mocks.requestCore).toHaveBeenNthCalledWith(2, "ai/generateCommitDraft", {
      previewId: "preview-1",
      provider,
      excludedPaths: ["private.json"],
      externalDisclosureConfirmed: true,
    });
    expect(JSON.stringify(mocks.requestCore.mock.calls)).not.toContain("apiKey");
  });
});

