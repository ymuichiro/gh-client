import { z } from "zod";

import type { CommandId } from "./commandIds";

export type CommandPermission = "viewer" | "write" | "admin";
export type UiExposure = "screen" | "console";

export type CommandCategory =
  | "dashboard"
  | "repositories"
  | "pull_requests"
  | "issues"
  | "actions"
  | "releases"
  | "settings"
  | "p2"
  | "console";

export type FieldType =
  | "text"
  | "textarea"
  | "number"
  | "boolean"
  | "select"
  | "string_list"
  | "json";

export interface FieldOption {
  label: string;
  value: string;
}

export interface CommandField {
  name: string;
  label: string;
  type: FieldType;
  required?: boolean;
  placeholder?: string;
  options?: FieldOption[];
  min?: number;
}

export interface CommandSpec {
  id: CommandId;
  title: string;
  description: string;
  category: CommandCategory;
  requiredPermission: CommandPermission;
  exposure: UiExposure;
  destructive: boolean;
  needsRepoContext: boolean;
  payloadSchema: z.ZodTypeAny;
  responseSchema: z.ZodTypeAny;
  fields: CommandField[];
}

export interface CommandEnvelope {
  contract_version: string;
  request_id: string;
  command_id: CommandId;
  permission?: CommandPermission;
  payload: Record<string, unknown>;
}

export interface FrontendInvokeError {
  code: string;
  message: string;
  retryable: boolean;
  fingerprint: string;
  request_id: string;
  command_id: string;
}

export interface CommandExecutionRecord {
  timestamp: string;
  requestId: string;
  commandId: CommandId;
  repo?: string;
  status: "success" | "error";
  code?: string;
}

export interface RepoContext {
  owner: string;
  repo: string;
  viewerPermission?: string;
}

export interface CommandSelectionOptions {
  ownerOptions: string[];
  repoOptions: string[];
  branchOptions: string[];
  pullRequestNumberOptions: number[];
  issueNumberOptions: number[];
  runIdOptions: number[];
  releaseTagOptions: string[];
}
