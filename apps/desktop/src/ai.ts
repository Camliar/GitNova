import type {
  AiCommitDraft,
  AiGenerateCommitDraftParams,
  AiInputPreview,
  AiInputPreviewParams,
  AiProviderConfig,
} from "@gitnova/protocol";
import { coreResult, requestCore } from "./core";

export async function previewAiInput(provider: AiProviderConfig, excludedPaths: string[]): Promise<AiInputPreview> {
  const params: AiInputPreviewParams = { provider, excludedPaths };
  return coreResult(await requestCore<AiInputPreview>("ai/inputPreview", params));
}

export async function generateAiCommitDraft(
  previewId: string,
  provider: AiProviderConfig,
  excludedPaths: string[],
  externalDisclosureConfirmed: boolean,
): Promise<AiCommitDraft> {
  const params: AiGenerateCommitDraftParams = {
    previewId,
    provider,
    excludedPaths,
    externalDisclosureConfirmed,
  };
  return coreResult(await requestCore<AiCommitDraft>("ai/generateCommitDraft", params));
}

