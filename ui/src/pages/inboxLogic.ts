export type SortMode = "priority" | "updated_desc" | "updated_asc";

export type ItemKind = "issue" | "pr";

export interface InboxFilters {
  query: string;
  state: "all" | "open" | "closed" | "merged";
  label: string;
  assignee: string;
  reviewer: string;
  draft: "all" | "yes" | "no";
  updatedWithinHours: string;
}

export interface InboxItem {
  id: string;
  kind: ItemKind;
  owner: string;
  repo: string;
  number: number;
  title: string;
  state: string;
  url: string;
  author: string | null;
  labels: string[];
  assignees: string[];
  reviewers: string[];
  isDraft: boolean;
  reviewDecision: string | null;
  updatedAt: string | null;
}

export type InboxActionType =
  | "close"
  | "reopen"
  | "comment"
  | "approve"
  | "request_changes"
  | "merge"
  | "issue_edit";

export type ActionGuardReason =
  | "permission_required"
  | "pr_only"
  | "issue_only"
  | "already_closed"
  | "already_open"
  | "pr_not_open"
  | "pr_merged"
  | "pr_draft";

export interface BatchExecutionResult {
  total: number;
  processed: number;
  success: number;
  failed: number;
  skipped: number;
}

export type BatchExecutionOutcome = "success" | "failed" | "skipped";

export interface ItemTiming {
  updatedTimestamp: number;
  ageHours: number | null;
  isStale: boolean;
}

export function initialInboxFilters(): InboxFilters {
  return {
    query: "",
    state: "all",
    label: "",
    assignee: "",
    reviewer: "",
    draft: "all",
    updatedWithinHours: "",
  };
}

export function filterAndSortItems(
  items: InboxItem[],
  filters: InboxFilters,
  sortMode: SortMode,
  slaHours: number,
  currentLogin?: string | null,
): InboxItem[] {
  const normalizedCurrentLogin = currentLogin?.trim().toLowerCase() ?? "";
  const resolveSelfAlias = (rawValue: string): string => {
    const trimmed = rawValue.trim();
    if (trimmed.toLowerCase() !== "@me") {
      return trimmed.toLowerCase();
    }

    return normalizedCurrentLogin;
  };

  const lowerQuery = filters.query.trim().toLowerCase();
  const lowerLabel = filters.label.trim().toLowerCase();
  const lowerAssignee = resolveSelfAlias(filters.assignee);
  const lowerReviewer = resolveSelfAlias(filters.reviewer);
  const withinHours = Number(filters.updatedWithinHours.trim());

  const filtered = items.filter((item) => {
    if (filters.state !== "all") {
      const normalizedState = normalizeState(item.state);
      if (filters.state === "merged") {
        if (!(item.kind === "pr" && normalizedState === "merged")) {
          return false;
        }
      } else if (normalizedState !== filters.state) {
        return false;
      }
    }

    if (filters.draft === "yes" && !item.isDraft) {
      return false;
    }

    if (filters.draft === "no" && item.isDraft) {
      return false;
    }

    if (lowerLabel && !item.labels.some((value) => value.toLowerCase().includes(lowerLabel))) {
      return false;
    }

    if (
      lowerAssignee &&
      !item.assignees.some((value) => value.toLowerCase().includes(lowerAssignee))
    ) {
      return false;
    }

    if (
      lowerReviewer &&
      !item.reviewers.some((value) => value.toLowerCase().includes(lowerReviewer))
    ) {
      return false;
    }

    if (Number.isFinite(withinHours) && withinHours > 0) {
      const timing = deriveItemTiming(item.updatedAt, slaHours);
      if (timing.ageHours === null || timing.ageHours > withinHours) {
        return false;
      }
    }

    if (!lowerQuery) {
      return true;
    }

    const haystack = [
      item.title,
      item.owner,
      item.repo,
      String(item.number),
      item.author ?? "",
      ...item.labels,
      ...item.assignees,
      ...item.reviewers,
    ]
      .join(" ")
      .toLowerCase();

    return haystack.includes(lowerQuery);
  });

  return [...filtered].sort((left, right) => compareItems(left, right, sortMode, slaHours));
}

export function compareItems(
  left: InboxItem,
  right: InboxItem,
  sortMode: SortMode,
  slaHours: number,
): number {
  if (sortMode === "updated_desc") {
    return byUpdatedAt(right, left);
  }

  if (sortMode === "updated_asc") {
    return byUpdatedAt(left, right);
  }

  const scoreLeft = priorityScore(left, slaHours);
  const scoreRight = priorityScore(right, slaHours);
  if (scoreLeft !== scoreRight) {
    return scoreRight - scoreLeft;
  }

  return byUpdatedAt(right, left);
}

export function priorityScore(item: InboxItem, slaHours: number): number {
  const timing = deriveItemTiming(item.updatedAt, slaHours);
  let score = 0;

  if (normalizeState(item.state) === "open") {
    score += 3;
  }

  if (item.kind === "pr" && item.reviewers.length > 0) {
    score += 4;
  }

  if (item.isDraft) {
    score -= 2;
  }

  if (timing.isStale) {
    score += 6;
  }

  if (timing.ageHours !== null) {
    score += Math.min(Math.floor(timing.ageHours / 12), 8);
  }

  return score;
}

export function deriveItemTiming(updatedAt: string | null, slaHours: number): ItemTiming {
  const updatedTimestamp = parseDate(updatedAt);
  if (updatedTimestamp <= 0) {
    return {
      updatedTimestamp: 0,
      ageHours: null,
      isStale: false,
    };
  }

  const diffMs = Date.now() - updatedTimestamp;
  const ageHours = diffMs / (1000 * 60 * 60);
  return {
    updatedTimestamp,
    ageHours,
    isStale: ageHours > slaHours,
  };
}

export function formatUpdatedLabel(updatedAt: string | null): string {
  const timing = deriveItemTiming(updatedAt, Number.MAX_SAFE_INTEGER);
  if (timing.updatedTimestamp <= 0) {
    return "-";
  }

  const date = new Date(timing.updatedTimestamp);
  const absolute = date.toLocaleString();
  const relative = formatAgeHours(timing.ageHours);
  return `${absolute} (${relative})`;
}

export function formatAgeHours(ageHours: number | null): string {
  if (ageHours === null || !Number.isFinite(ageHours) || ageHours < 0) {
    return "-";
  }

  if (ageHours < 1) {
    return "<1h";
  }

  if (ageHours < 24) {
    return `${Math.floor(ageHours)}h`;
  }

  const days = Math.floor(ageHours / 24);
  const hours = Math.floor(ageHours % 24);
  if (hours === 0) {
    return `${days}d`;
  }

  return `${days}d ${hours}h`;
}

export function evaluateActionGuard(
  item: InboxItem,
  permission: "viewer" | "write" | "admin" | null,
  action: InboxActionType,
): ActionGuardReason | null {
  const writable = permission === "write" || permission === "admin";
  if (!writable) {
    return "permission_required";
  }

  const state = normalizeState(item.state);

  switch (action) {
    case "close":
      return state === "open" ? null : "already_closed";
    case "reopen":
      if (item.kind === "pr" && state === "merged") {
        return "pr_merged";
      }
      return state === "closed" ? null : "already_open";
    case "comment":
      return null;
    case "approve":
    case "request_changes":
      if (item.kind !== "pr") {
        return "pr_only";
      }
      if (state !== "open") {
        return "pr_not_open";
      }
      return null;
    case "merge":
      if (item.kind !== "pr") {
        return "pr_only";
      }
      if (state === "merged") {
        return "pr_merged";
      }
      if (state !== "open") {
        return "pr_not_open";
      }
      if (item.isDraft) {
        return "pr_draft";
      }
      return null;
    case "issue_edit":
      return item.kind === "issue" ? null : "issue_only";
    default:
      return null;
  }
}

export function selectBatchCloseTargets(items: InboxItem[]): InboxItem[] {
  return items.filter((item) => normalizeState(item.state) === "open");
}

export function createBatchExecutionResult(total: number): BatchExecutionResult {
  return {
    total,
    processed: 0,
    success: 0,
    failed: 0,
    skipped: 0,
  };
}

export function appendBatchExecutionResult(
  result: BatchExecutionResult,
  outcome: BatchExecutionOutcome,
): BatchExecutionResult {
  return {
    ...result,
    processed: result.processed + 1,
    success: result.success + (outcome === "success" ? 1 : 0),
    failed: result.failed + (outcome === "failed" ? 1 : 0),
    skipped: result.skipped + (outcome === "skipped" ? 1 : 0),
  };
}

export function normalizeMergeMethod(value: string | null): "merge" | "squash" | "rebase" {
  const normalized = (value ?? "squash").trim().toLowerCase();
  if (normalized === "merge" || normalized === "squash" || normalized === "rebase") {
    return normalized;
  }

  return "squash";
}

export function parseCommaSeparated(raw: string): string[] {
  return raw
    .split(",")
    .map((value) => value.trim())
    .filter((value) => value.length > 0);
}

export function extractReviewerLogins(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }

  const reviewers: string[] = [];
  for (const entry of value) {
    const record = asRecord(entry);
    const requestedReviewer = asRecord(record.requestedReviewer);

    const directLogin = asString(record.login);
    const directName = asString(record.name);
    const nestedLogin = asString(requestedReviewer.login);
    const nestedName = asString(requestedReviewer.name);

    const resolved = directLogin ?? nestedLogin ?? directName ?? nestedName;
    if (resolved) {
      reviewers.push(resolved);
    }
  }

  return Array.from(new Set(reviewers));
}

export function parsePositiveNumber(value: string): number | null {
  const parsed = Number(value.trim());
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return null;
  }

  return parsed;
}

export function toRepoKey(owner: string, repo: string): string {
  return `${owner}/${repo}`;
}

export function parseDate(value: string | null): number {
  if (!value) {
    return 0;
  }

  const timestamp = Date.parse(value);
  if (Number.isNaN(timestamp)) {
    return 0;
  }

  return timestamp;
}

export function normalizeState(value: string): string {
  return value.trim().toLowerCase();
}

export function asRecord(value: unknown): Record<string, unknown> {
  if (typeof value === "object" && value !== null) {
    return value as Record<string, unknown>;
  }

  return {};
}

export function asString(value: unknown): string | null {
  if (typeof value === "string" && value.trim().length > 0) {
    return value.trim();
  }

  return null;
}

export function asNumber(value: unknown): number | null {
  if (typeof value === "number" && Number.isFinite(value)) {
    return value;
  }

  if (typeof value === "string") {
    const parsed = Number(value);
    if (Number.isFinite(parsed)) {
      return parsed;
    }
  }

  return null;
}

export function extractNamedValues(value: unknown, key: string): string[] {
  if (!Array.isArray(value)) {
    return [];
  }

  const values = value
    .map((item) => asString(asRecord(item)[key]))
    .filter((item): item is string => Boolean(item));

  return Array.from(new Set(values));
}

function byUpdatedAt(left: InboxItem, right: InboxItem): number {
  const leftTs = parseDate(left.updatedAt);
  const rightTs = parseDate(right.updatedAt);
  return leftTs - rightTs;
}
