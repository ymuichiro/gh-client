import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  appendBatchExecutionResult,
  createBatchExecutionResult,
  deriveItemTiming,
  evaluateActionGuard,
  extractReviewerLogins,
  filterAndSortItems,
  formatAgeHours,
  initialInboxFilters,
  normalizeMergeMethod,
  selectBatchCloseTargets,
  type InboxItem,
} from "./inboxLogic";

function buildItem(overrides: Partial<InboxItem>): InboxItem {
  return {
    id: "pr:acme/repo#1",
    kind: "pr",
    owner: "acme",
    repo: "repo",
    number: 1,
    title: "sample",
    state: "OPEN",
    url: "https://example.test/pr/1",
    author: "alice",
    labels: [],
    assignees: [],
    reviewers: [],
    isDraft: false,
    reviewDecision: null,
    updatedAt: "2026-03-08T00:00:00Z",
    ...overrides,
  };
}

describe("inboxLogic", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-03-08T12:00:00Z"));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("extracts and deduplicates reviewer logins", () => {
    const reviewers = extractReviewerLogins([
      { requestedReviewer: { login: "reviewer-a" } },
      { login: "reviewer-b" },
      { name: "team-reviewers" },
      { requestedReviewer: { name: "team-reviewers" } },
      { requestedReviewer: { login: "reviewer-a" } },
    ]);

    expect(reviewers).toEqual(["reviewer-a", "reviewer-b", "team-reviewers"]);
  });

  it("filters and sorts by priority with SLA staleness", () => {
    const stalePr = buildItem({
      id: "pr:acme/repo#10",
      number: 10,
      title: "stale review",
      reviewers: ["maintainer"],
      updatedAt: "2026-03-06T00:00:00Z",
    });

    const freshIssue = buildItem({
      id: "issue:acme/repo#20",
      kind: "issue",
      number: 20,
      title: "fresh bug",
      state: "OPEN",
      reviewers: [],
      updatedAt: "2026-03-08T11:30:00Z",
    });

    const mergedPr = buildItem({
      id: "pr:acme/repo#30",
      number: 30,
      title: "merged item",
      state: "MERGED",
      reviewers: ["maintainer"],
      updatedAt: "2026-03-08T10:00:00Z",
    });

    const filters = {
      ...initialInboxFilters(),
      state: "open" as const,
      reviewer: "maintainer",
    };

    const result = filterAndSortItems([freshIssue, mergedPr, stalePr], filters, "priority", 24);

    expect(result).toHaveLength(1);
    expect(result[0]?.id).toBe(stalePr.id);
  });

  it("derives age and stale flag from updatedAt", () => {
    const timing = deriveItemTiming("2026-03-07T00:00:00Z", 24);

    expect(timing.ageHours).toBe(36);
    expect(timing.isStale).toBe(true);
    expect(formatAgeHours(timing.ageHours)).toBe("1d 12h");
  });

  it("normalizes merge method", () => {
    expect(normalizeMergeMethod("merge")).toBe("merge");
    expect(normalizeMergeMethod(" REBASE ")).toBe("rebase");
    expect(normalizeMergeMethod("invalid")).toBe("squash");
    expect(normalizeMergeMethod(null)).toBe("squash");
  });

  it("evaluates action guards by permission and state", () => {
    const mergedPr = buildItem({ state: "MERGED" });
    const draftPr = buildItem({ isDraft: true });
    const issue = buildItem({ kind: "issue", id: "issue:acme/repo#4" });

    expect(evaluateActionGuard(mergedPr, "write", "merge")).toBe("pr_merged");
    expect(evaluateActionGuard(draftPr, "admin", "merge")).toBe("pr_draft");
    expect(evaluateActionGuard(issue, "write", "approve")).toBe("pr_only");
    expect(evaluateActionGuard(issue, "viewer", "comment")).toBe("permission_required");
  });

  it("selects only open items for batch close", () => {
    const items: InboxItem[] = [
      buildItem({ id: "pr:acme/repo#1", state: "OPEN" }),
      buildItem({ id: "pr:acme/repo#2", state: "CLOSED" }),
      buildItem({ id: "pr:acme/repo#3", state: "MERGED" }),
    ];

    expect(selectBatchCloseTargets(items).map((item) => item.id)).toEqual(["pr:acme/repo#1"]);
  });

  it("aggregates batch execution results", () => {
    let summary = createBatchExecutionResult(3);
    summary = appendBatchExecutionResult(summary, "success");
    summary = appendBatchExecutionResult(summary, "failed");
    summary = appendBatchExecutionResult(summary, "skipped");

    expect(summary).toEqual({
      total: 3,
      processed: 3,
      success: 1,
      failed: 1,
      skipped: 1,
    });
  });
});
