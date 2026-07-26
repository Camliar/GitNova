import { invoke, isTauri } from "@tauri-apps/api/core";

export interface DiagnosticInfo {
  path: string;
  maxBytes: number;
  retainedFiles: number;
}

export async function getDiagnosticInfo(): Promise<DiagnosticInfo | null> {
  return isTauri() ? invoke<DiagnosticInfo>("diagnostics_info") : null;
}
