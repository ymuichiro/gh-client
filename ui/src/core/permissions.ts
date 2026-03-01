import type { CommandPermission } from "./types";

export function normalizeViewerPermission(value?: string): CommandPermission | null {
  if (!value) {
    return null;
  }

  const normalized = value.trim().toUpperCase();
  if (normalized === "ADMIN" || normalized === "MAINTAIN") {
    return "admin";
  }

  if (normalized === "WRITE" || normalized === "TRIAGE") {
    return "write";
  }

  if (normalized === "READ") {
    return "viewer";
  }

  return null;
}

export function resolveEnvelopePermission(
  required: CommandPermission,
  repoPermission: CommandPermission | null,
  needsRepoContext: boolean,
): CommandPermission | null {
  if (!needsRepoContext) {
    return required;
  }

  if (required === "viewer") {
    return "viewer";
  }

  if (!repoPermission) {
    return null;
  }

  if (required === "write") {
    return repoPermission === "write" || repoPermission === "admin" ? "write" : null;
  }

  return repoPermission === "admin" ? "admin" : null;
}
