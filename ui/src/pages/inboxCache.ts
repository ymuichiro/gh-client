import type { InboxItem, ItemKind } from "./inboxLogic";

export interface InboxCacheEntry {
  items: InboxItem[];
  warnings: string[];
  updatedAt: number;
}

export const INBOX_CACHE_TTL_MS = 5 * 60 * 1000;

const STORAGE_PREFIX = "gh-client-inbox-cache-v1";
const memoryCache = new Map<string, InboxCacheEntry>();

export function readInboxCache(mode: ItemKind, repoKeys: string[]): InboxCacheEntry | null {
  const cacheKey = buildCacheKey(mode, repoKeys);
  const fromMemory = memoryCache.get(cacheKey);
  if (fromMemory) {
    return fromMemory;
  }

  const storage = getSafeStorage();
  if (!storage) {
    return null;
  }

  try {
    const raw = storage.getItem(`${STORAGE_PREFIX}:${cacheKey}`);
    if (!raw) {
      return null;
    }

    const parsed = JSON.parse(raw) as Partial<InboxCacheEntry>;
    if (
      !parsed ||
      !Array.isArray(parsed.items) ||
      !Array.isArray(parsed.warnings) ||
      typeof parsed.updatedAt !== "number"
    ) {
      return null;
    }

    const entry: InboxCacheEntry = {
      items: parsed.items as InboxItem[],
      warnings: parsed.warnings
        .map((value) => (typeof value === "string" ? value : ""))
        .filter((value) => value.length > 0),
      updatedAt: parsed.updatedAt,
    };
    memoryCache.set(cacheKey, entry);
    return entry;
  } catch (_cause) {
    return null;
  }
}

export function writeInboxCache(
  mode: ItemKind,
  repoKeys: string[],
  entry: InboxCacheEntry,
): void {
  const cacheKey = buildCacheKey(mode, repoKeys);
  memoryCache.set(cacheKey, entry);

  const storage = getSafeStorage();
  if (!storage) {
    return;
  }

  try {
    storage.setItem(`${STORAGE_PREFIX}:${cacheKey}`, JSON.stringify(entry));
  } catch (_cause) {
    // Ignore storage write errors.
  }
}

export function isInboxCacheStale(entry: InboxCacheEntry, now = Date.now()): boolean {
  return now - entry.updatedAt >= INBOX_CACHE_TTL_MS;
}

function buildCacheKey(mode: ItemKind, repoKeys: string[]): string {
  return `${mode}:${[...repoKeys].sort((left, right) => left.localeCompare(right)).join("|")}`;
}

function getSafeStorage(): Storage | null {
  if (typeof window === "undefined") {
    return null;
  }

  const storage = window.localStorage;
  if (
    storage &&
    typeof storage.getItem === "function" &&
    typeof storage.setItem === "function"
  ) {
    return storage;
  }

  return null;
}
