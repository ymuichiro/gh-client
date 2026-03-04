import type { CommandExecutionRecord } from "./types";

const STORAGE_KEY = "gh-client-history";
const MAX_ITEMS = 200;

export function loadHistory(): CommandExecutionRecord[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return [];
    }

    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) {
      return [];
    }

    return parsed as CommandExecutionRecord[];
  } catch (_err) {
    return [];
  }
}

export function appendHistory(entry: CommandExecutionRecord): CommandExecutionRecord[] {
  const current = loadHistory();
  const updated = [entry, ...current].slice(0, MAX_ITEMS);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
  return updated;
}

export function clearHistory(): void {
  localStorage.removeItem(STORAGE_KEY);
}
