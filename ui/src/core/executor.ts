import { v4 as uuidv4 } from "uuid";

import { CONTRACT_VERSION, type CommandId } from "./commandIds";
import { COMMAND_CATALOG } from "./commandCatalog";
import { executeMockEnvelope } from "./mockExecutor";
import type { CommandEnvelope, CommandPermission, FrontendInvokeError } from "./types";

export interface ExecuteCommandOptions {
  permission?: CommandPermission;
}

export interface ExecuteCommandResult<T = unknown> {
  requestId: string;
  commandId: CommandId;
  payload: Record<string, unknown>;
  data: T;
}

export class ExecutionError extends Error {
  readonly detail: FrontendInvokeError;

  constructor(detail: FrontendInvokeError) {
    super(detail.message);
    this.detail = detail;
  }
}

const executionMode = (import.meta.env.VITE_EXECUTION_MODE || "tauri") as
  | "tauri"
  | "mock"
  | "bridge";
const bridgeUrl = import.meta.env.VITE_EXECUTION_BRIDGE_URL as string | undefined;

export async function executeCommand<T = unknown>(
  commandId: CommandId,
  payload: Record<string, unknown>,
  options: ExecuteCommandOptions = {},
): Promise<ExecuteCommandResult<T>> {
  const spec = COMMAND_CATALOG[commandId];
  const parsedPayload = spec.payloadSchema.parse(payload) as Record<string, unknown>;

  const requestId = typeof crypto !== "undefined" && crypto.randomUUID ? crypto.randomUUID() : uuidv4();

  const envelope: CommandEnvelope = {
    contract_version: CONTRACT_VERSION,
    request_id: requestId,
    command_id: commandId,
    permission: options.permission,
    payload: parsedPayload,
  };

  const raw = await executeEnvelope(envelope);
  const data = spec.responseSchema.parse(raw) as T;

  return {
    requestId,
    commandId,
    payload: parsedPayload,
    data,
  };
}

async function executeEnvelope(envelope: CommandEnvelope): Promise<unknown> {
  if (executionMode === "mock") {
    return executeMockEnvelope(envelope);
  }

  if (executionMode === "bridge") {
    const url = bridgeUrl ?? "http://127.0.0.1:8787/execute";
    const response = await fetchBridge(url, envelope);

    if (!response.ok) {
      const detail = (await response.json()) as FrontendInvokeError;
      throw new ExecutionError(detail);
    }

    return response.json();
  }

  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke("frontend_execute", { envelope });
  } catch (error) {
    if (hasTauriRuntime()) {
      throw new ExecutionError(normalizeInvokeError(error, envelope));
    }

    return executeMockEnvelope(envelope);
  }
}

function hasTauriRuntime(): boolean {
  const scopedWindow = window as unknown as { __TAURI_INTERNALS__?: unknown };
  return Boolean(scopedWindow.__TAURI_INTERNALS__);
}

function normalizeInvokeError(error: unknown, envelope: CommandEnvelope): FrontendInvokeError {
  if (isObject(error) && "code" in error && "message" in error) {
    return {
      code: asString(error.code, "execution_error"),
      message: asString(error.message, "unknown invoke error"),
      retryable: Boolean(error.retryable),
      fingerprint: asString(error.fingerprint, "invoke"),
      request_id: asString(error.request_id, envelope.request_id),
      command_id: asString(error.command_id, envelope.command_id),
    };
  }

  return {
    code: "execution_error",
    message: error instanceof Error ? error.message : String(error),
    retryable: false,
    fingerprint: "invoke-failure",
    request_id: envelope.request_id,
    command_id: envelope.command_id,
  };
}

async function fetchBridge(url: string, envelope: CommandEnvelope): Promise<Response> {
  let attempt = 0;

  while (true) {
    attempt += 1;

    try {
      return await fetch(url, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(envelope),
      });
    } catch (error) {
      if (attempt >= 12) {
        throw error;
      }
      await new Promise((resolvePromise) => setTimeout(resolvePromise, 500));
    }
  }
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function asString(value: unknown, fallback: string): string {
  if (typeof value === "string" && value.length > 0) {
    return value;
  }

  return fallback;
}
