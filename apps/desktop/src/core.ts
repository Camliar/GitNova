import { invoke, isTauri } from "@tauri-apps/api/core";
import type { ServerCapabilities } from "@gitnova/protocol";

export interface CoreStatus {
  connected: boolean;
  protocolVersion: string | null;
  capabilities: ServerCapabilities | null;
}

export interface DesktopError {
  code: string;
  message: string;
  retryable: boolean;
}

export interface CoreResponse<T = unknown> {
  jsonrpc: "2.0";
  id: number | string | null;
  result?: T;
  error?: {
    code: number;
    message: string;
    data: { stableCode: string; retryable: boolean; details?: Record<string, unknown> };
  };
}

export function coreResult<T>(response: CoreResponse<T>): T {
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

const stopped: CoreStatus = {
  connected: false,
  protocolVersion: null,
  capabilities: null,
};

export async function getCoreStatus(): Promise<CoreStatus> {
  return isTauri() ? invoke<CoreStatus>("core_status") : stopped;
}

export async function startCore(): Promise<CoreStatus> {
  if (!isTauri()) {
    throw {
      code: "desktop.preview_only",
      message: "Core can only be started by the native Desktop Host",
      retryable: false,
    } satisfies DesktopError;
  }
  return invoke<CoreStatus>("core_start");
}

export async function requestCore<T>(method: string, params: unknown): Promise<CoreResponse<T>> {
  return invoke<CoreResponse<T>>("core_request", { method, params });
}

export async function shutdownCore(): Promise<CoreStatus> {
  return invoke<CoreStatus>("core_shutdown");
}

export function asDesktopError(error: unknown): DesktopError {
  if (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    "message" in error &&
    "retryable" in error &&
    typeof error.code === "string" &&
    typeof error.message === "string" &&
    typeof error.retryable === "boolean"
  ) {
    return { code: error.code, message: error.message, retryable: error.retryable };
  }
  return {
    code: "desktop.unknown_error",
    message: "Desktop Host could not complete the request",
    retryable: true,
  };
}
