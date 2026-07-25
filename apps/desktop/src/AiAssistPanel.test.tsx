import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AiAssistPanel } from "./AiAssistPanel";

const ai = vi.hoisted(() => ({ previewAiInput: vi.fn(), generateAiCommitDraft: vi.fn() }));
vi.mock("./ai", () => ai);

const preview = {
  previewId: "preview-1",
  indexFingerprint: "a".repeat(64),
  providerKind: "openAi" as const,
  model: "user-model",
  destination: "external" as const,
  endpoint: "https://api.openai.com/v1/responses",
  files: [
    { path: "src/app.ts", additions: 4, deletions: 1, patchBytes: 512, state: "included" as const, reason: null },
    { path: ".env", additions: 1, deletions: 0, patchBytes: 0, state: "excluded" as const, reason: "excluded by sensitive-path policy" },
  ],
  stagedDiffBytes: 512,
  promptBytes: 768,
  truncated: false,
  externalConfirmationRequired: true,
};

describe("Desktop AI Assist", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    ai.previewAiInput.mockResolvedValue(preview);
    ai.generateAiCommitDraft.mockResolvedValue({
      previewId: "preview-1",
      providerKind: "openAi",
      model: "user-model",
      commitMessage: "feat: improve AI disclosure",
      suggestions: [{ kind: "runTests", title: "Run tests", detail: "Verify the staged behavior.", affectedPaths: ["src/app.ts"] }],
      warnings: ["Review generated wording before committing."],
    });
  });

  it("requires external disclosure confirmation and hands off an editable draft without committing", async () => {
    const onUse = vi.fn();
    render(<AiAssistPanel onUseCommitMessage={onUse} />);
    fireEvent.change(screen.getByLabelText("Provider"), { target: { value: "openAi" } });
    fireEvent.change(screen.getByLabelText("Model"), { target: { value: "user-model" } });
    fireEvent.change(screen.getByLabelText(/Excluded repository paths/), { target: { value: "private.json\nprivate.json" } });
    expect(screen.queryByLabelText(/API key/i)).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Preview input" }));

    expect(await screen.findByRole("heading", { name: "Leaves this environment" })).toBeInTheDocument();
    expect(screen.getByText(".env")).toBeInTheDocument();
    expect(screen.getByText(/sensitive-path policy/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Generate draft" })).toBeDisabled();
    fireEvent.click(screen.getByLabelText(/I confirm the listed staged patch/));
    fireEvent.click(screen.getByRole("button", { name: "Generate draft" }));

    const message = await screen.findByLabelText("Commit message");
    expect(message).toHaveValue("feat: improve AI disclosure");
    expect(screen.getByText("Suggestions — never run automatically")).toBeInTheDocument();
    fireEvent.change(message, { target: { value: "feat: edited by the user" } });
    fireEvent.click(screen.getByRole("button", { name: "Use in commit" }));
    expect(onUse).toHaveBeenCalledWith("feat: edited by the user");
    expect(screen.getByRole("status")).toHaveTextContent("confirm the Git action separately");
    expect(ai.generateAiCommitDraft).toHaveBeenCalledWith(
      "preview-1",
      { kind: "openAi", model: "user-model" },
      ["private.json"],
      true,
    );
  });

  it("invalidates disclosure and confirmation when configuration changes", async () => {
    render(<AiAssistPanel onUseCommitMessage={vi.fn()} />);
    fireEvent.change(screen.getByLabelText("Provider"), { target: { value: "openAi" } });
    fireEvent.change(screen.getByLabelText("Model"), { target: { value: "user-model" } });
    fireEvent.click(screen.getByRole("button", { name: "Preview input" }));
    expect(await screen.findByRole("heading", { name: "Leaves this environment" })).toBeInTheDocument();
    fireEvent.click(screen.getByLabelText(/I confirm the listed staged patch/));
    fireEvent.change(screen.getByLabelText("Model"), { target: { value: "another-model" } });

    expect(screen.queryByRole("heading", { name: "Leaves this environment" })).not.toBeInTheDocument();
    expect(screen.queryByLabelText(/I confirm the listed staged patch/)).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Generate draft" })).not.toBeInTheDocument();
  });
});

