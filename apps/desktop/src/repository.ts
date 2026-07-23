import { open } from "@tauri-apps/plugin-dialog";
import type { RepositoryDescriptor, RepositoryPathParams } from "@gitnova/protocol";
import { requestCore, type DesktopError } from "./core";

export async function selectRepositoryDirectory(): Promise<string | null> {
  const selected = await open({ directory: true, multiple: false });
  return typeof selected === "string" ? selected : null;
}

export async function openRepository(path: string): Promise<RepositoryDescriptor> {
  const params: RepositoryPathParams = { path };
  const response = await requestCore<RepositoryDescriptor>("repository/open", params);
  if (response.error) {
    throw {
      code: response.error.data.stableCode,
      message: response.error.message,
      retryable: response.error.data.retryable,
    } satisfies DesktopError;
  }
  if (!("result" in response) || response.result === undefined) {
    throw {
      code: "desktop.core_protocol_failed",
      message: "GitNova Core returned an invalid protocol response",
      retryable: false,
    } satisfies DesktopError;
  }
  return response.result;
}
