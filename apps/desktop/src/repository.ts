import { open } from "@tauri-apps/plugin-dialog";
import type { RepositoryDescriptor, RepositoryPathParams } from "@gitnova/protocol";
import { coreResult, requestCore } from "./core";

export async function selectRepositoryDirectory(): Promise<string | null> {
  const selected = await open({ directory: true, multiple: false });
  return typeof selected === "string" ? selected : null;
}

export async function openRepository(path: string): Promise<RepositoryDescriptor> {
  const params: RepositoryPathParams = { path };
  return coreResult(await requestCore<RepositoryDescriptor>("repository/open", params));
}
