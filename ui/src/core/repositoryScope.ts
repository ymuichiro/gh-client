import { normalizeViewerPermission } from "./permissions";
import type { CommandPermission } from "./types";

const STORAGE_KEY = "gh-client-repository-scope";

export interface ScopedRepositoryTarget {
  owner: string;
  repo: string;
  viewerPermission: CommandPermission | null;
}

export interface RepositoryScopeConfig {
  orgs: string[];
  repositories: ScopedRepositoryTarget[];
  updatedAt: string;
}

export function loadRepositoryScope(): RepositoryScopeConfig | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return null;
    }

    const parsed = JSON.parse(raw) as unknown;
    const record = asRecord(parsed);

    const orgs = dedupeStrings(
      Array.isArray(record.orgs)
        ? record.orgs
            .map((value) => asString(value))
            .filter((value): value is string => value !== null)
        : [],
    );

    const repositories = parseRepositories(record.repositories);
    if (repositories.length === 0) {
      return null;
    }

    return {
      orgs,
      repositories,
      updatedAt: asString(record.updatedAt) ?? new Date().toISOString(),
    };
  } catch (_cause) {
    return null;
  }
}

export function saveRepositoryScope(config: RepositoryScopeConfig): void {
  const normalized: RepositoryScopeConfig = {
    orgs: dedupeStrings(config.orgs),
    repositories: dedupeRepositories(config.repositories),
    updatedAt: config.updatedAt,
  };

  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(normalized));
  } catch (_cause) {
    // ignore persistence failures in restricted runtime contexts
  }
}

export function clearRepositoryScope(): void {
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch (_cause) {
    // ignore persistence failures in restricted runtime contexts
  }
}

export function toRepositoryKey(owner: string, repo: string): string {
  return `${owner}/${repo}`;
}

export function dedupeRepositories(
  repositories: ScopedRepositoryTarget[],
): ScopedRepositoryTarget[] {
  const seen = new Set<string>();
  const next: ScopedRepositoryTarget[] = [];

  for (const repository of repositories) {
    const owner = repository.owner.trim();
    const repo = repository.repo.trim();
    if (!owner || !repo) {
      continue;
    }

    const key = toRepositoryKey(owner, repo);
    if (seen.has(key)) {
      continue;
    }

    seen.add(key);
    next.push({
      owner,
      repo,
      viewerPermission: repository.viewerPermission,
    });
  }

  return next;
}

export function groupRepositoriesByOwner(
  repositories: ScopedRepositoryTarget[],
): Record<string, ScopedRepositoryTarget[]> {
  const grouped: Record<string, ScopedRepositoryTarget[]> = {};

  for (const repository of dedupeRepositories(repositories)) {
    if (!grouped[repository.owner]) {
      grouped[repository.owner] = [];
    }

    grouped[repository.owner].push(repository);
  }

  for (const owner of Object.keys(grouped)) {
    grouped[owner].sort((left, right) => left.repo.localeCompare(right.repo));
  }

  return grouped;
}

function parseRepositories(value: unknown): ScopedRepositoryTarget[] {
  if (!Array.isArray(value)) {
    return [];
  }

  const repositories: ScopedRepositoryTarget[] = [];
  for (const entry of value) {
    const record = asRecord(entry);
    const owner = asString(record.owner);
    const repo = asString(record.repo);

    if (!owner || !repo) {
      continue;
    }

    repositories.push({
      owner,
      repo,
      viewerPermission: normalizeViewerPermission(asString(record.viewerPermission) ?? undefined),
    });
  }

  return dedupeRepositories(repositories);
}

function dedupeStrings(values: string[]): string[] {
  const seen = new Set<string>();
  const next: string[] = [];

  for (const value of values) {
    const trimmed = value.trim();
    if (!trimmed || seen.has(trimmed)) {
      continue;
    }

    seen.add(trimmed);
    next.push(trimmed);
  }

  return next;
}

function asRecord(value: unknown): Record<string, unknown> {
  if (typeof value === "object" && value !== null) {
    return value as Record<string, unknown>;
  }

  return {};
}

function asString(value: unknown): string | null {
  if (typeof value === "string" && value.trim().length > 0) {
    return value.trim();
  }

  return null;
}
