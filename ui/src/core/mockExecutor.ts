import type { CommandEnvelope } from "./types";
import type { CommandId } from "./commandIds";

const mockNow = new Date().toISOString();
const mockState = createInitialMockState();
let nextMockIssueCommentId = 2000;

export async function executeMockEnvelope(envelope: CommandEnvelope): Promise<unknown> {
  switch (envelope.command_id) {
    case "auth.organizations.list":
      return [
        { login: "mock-org", name: "Mock Organization" },
        { login: "sandbox-team", name: "Sandbox Team" },
      ];

    case "auth.status":
      return {
        logged_in: true,
        account: "mock-user",
        host: "github.com",
        scopes: ["repo", "read:org"],
      };

    case "repo.list":
      return [
        {
          name: "gh-client",
          nameWithOwner: `${stringValue(envelope.payload.owner, "mock-user")}/gh-client`,
          description: "mock repository",
          url: "https://github.com/mock-user/gh-client",
          isPrivate: false,
          viewerPermission: "ADMIN",
        },
      ];

    case "pr.list":
      return [
        {
          number: 1,
          title: "Mock pull request",
          state: "OPEN",
          url: "https://github.com/mock-user/gh-client/pull/1",
          isDraft: false,
          author: { login: "review-bot" },
          headRefName: "feature/mock",
          baseRefName: "main",
          labels: [{ name: "needs-review" }],
          assignees: [{ login: "mock-user" }],
          reviewDecision: "REVIEW_REQUIRED",
          reviewRequests: [{ requestedReviewer: { login: "maintainer-a" } }],
          updatedAt: mockNow,
        },
        {
          number: 2,
          title: "Draft: polish inbox layout",
          state: "OPEN",
          url: "https://github.com/mock-user/gh-client/pull/2",
          isDraft: true,
          author: { login: "designer-b" },
          headRefName: "draft/layout",
          baseRefName: "main",
          labels: [{ name: "ui" }],
          assignees: [],
          reviewDecision: null,
          reviewRequests: [],
          updatedAt: mockNow,
        },
      ];

    case "issue.list":
      return ensureMockIssues(envelope.payload).map((issue) => ({
        number: issue.number,
        title: issue.title,
        state: issue.state,
        url: issue.url,
        author: issue.author,
        labels: issue.labels,
        assignees: issue.assignees,
        updatedAt: issue.updatedAt,
      }));

    case "issue.view": {
      const owner = stringValue(envelope.payload.owner, "mock-user");
      const repo = stringValue(envelope.payload.repo, "gh-client");
      const number = numberValue(envelope.payload.number, 1);
      const issue = ensureMockIssues(envelope.payload).find((entry) => entry.number === number);
      if (!issue) {
        throw new Error(`issue not found: ${owner}/${repo}#${number}`);
      }

      return {
        number: issue.number,
        title: issue.title,
        state: issue.state,
        url: issue.url,
        body: issue.body,
        author: issue.author,
        labels: issue.labels,
        assignees: issue.assignees,
        updatedAt: issue.updatedAt,
        comments: issue.comments,
      };
    }

    case "issue.close": {
      const number = numberValue(envelope.payload.number, 0);
      const issue = ensureMockIssues(envelope.payload).find((entry) => entry.number === number);
      if (issue) {
        issue.state = "CLOSED";
        issue.updatedAt = new Date().toISOString();
      }
      return {
        ok: true,
        mocked: true,
      };
    }

    case "issue.reopen": {
      const number = numberValue(envelope.payload.number, 0);
      const issue = ensureMockIssues(envelope.payload).find((entry) => entry.number === number);
      if (issue) {
        issue.state = "OPEN";
        issue.updatedAt = new Date().toISOString();
      }
      return {
        ok: true,
        mocked: true,
      };
    }

    case "issue.comment": {
      const number = numberValue(envelope.payload.number, 0);
      const body = stringValue(envelope.payload.body, "").trim();
      const owner = stringValue(envelope.payload.owner, "mock-user");
      const repo = stringValue(envelope.payload.repo, "gh-client");
      const issue = ensureMockIssues(envelope.payload).find((entry) => entry.number === number);
      if (issue && body.length > 0) {
        const commentId = nextMockIssueCommentId;
        nextMockIssueCommentId += 1;
        const createdAt = new Date().toISOString();
        issue.comments.push({
          id: commentId,
          body,
          createdAt,
          author: { login: "mock-user" },
          url: `https://github.com/${owner}/${repo}/issues/${number}#issuecomment-${commentId}`,
        });
        issue.updatedAt = createdAt;
      }
      return {
        ok: true,
        mocked: true,
      };
    }

    case "issue.edit": {
      const number = numberValue(envelope.payload.number, 0);
      const issue = ensureMockIssues(envelope.payload).find((entry) => entry.number === number);
      if (issue) {
        applyMockIssueEdit(issue, envelope.payload);
        issue.updatedAt = new Date().toISOString();
      }
      return {
        ok: true,
        mocked: true,
      };
    }

    case "run.list":
      return [
        {
          databaseId: 1,
          name: "ci",
          status: "completed",
          conclusion: "success",
          createdAt: mockNow,
        },
      ];

    case "release.list":
      return [
        {
          name: "v0.1.0",
          tagName: "v0.1.0",
          url: "https://github.com/mock-user/gh-client/releases/tag/v0.1.0",
          isDraft: false,
          isPrerelease: false,
        },
      ];

    case "pr.diff.raw.get":
      return {
        text: "diff --git a/README.md b/README.md\n+mock line",
      };

    case "pr.view":
      return {
        number: 1,
        title: "Mock pull request",
        body: "This PR updates the inbox flow.",
        state: "OPEN",
        url: "https://github.com/mock-user/gh-client/pull/1",
        isDraft: false,
        headRefName: "feature/mock",
        baseRefName: "main",
        mergeStateStatus: "CLEAN",
        reviewDecision: "REVIEW_REQUIRED",
        additions: 8,
        deletions: 2,
        changedFiles: 2,
      };

    case "pr.comments.list":
      return [
        {
          id: 9001,
          kind: "issue_comment",
          body: "Please double check the empty state copy.",
          created_at: mockNow,
          author: { login: "maintainer-a" },
        },
      ];

    case "pr.review_threads.list":
      return [
        {
          thread_id: "THREAD_MOCK_1",
          is_resolved: false,
          is_outdated: false,
          path: "ui/src/pages/InboxPage.tsx",
          line: 120,
          comments: [
            {
              id: 9101,
              kind: "review_comment",
              body: "Could we avoid duplicate fetches here?",
              created_at: mockNow,
              author: { login: "reviewer-a" },
            },
          ],
        },
        {
          thread_id: "THREAD_MOCK_2",
          is_resolved: true,
          is_outdated: false,
          path: "ui/src/styles.css",
          line: 50,
          comments: [],
        },
      ];

    case "pr.diff.files.list":
      return [
        {
          filename: "ui/src/pages/InboxPage.tsx",
          status: "modified",
          additions: 6,
          deletions: 2,
          changes: 8,
          patch: "@@ -120,6 +120,10 @@\n+const requestSeq = ++prDetailFetchSeq.current;\n+if (requestSeq !== prDetailFetchSeq.current) return;",
        },
        {
          filename: "README.md",
          status: "modified",
          additions: 1,
          deletions: 0,
          changes: 1,
          patch: "@@ -1 +1,2 @@\n+mock line",
        },
      ];

    default:
      return {
        ok: true,
        mocked: true,
        command_id: envelope.command_id,
        payload: envelope.payload,
      };
  }
}

export function inferMockPermission(commandId: CommandId): "viewer" | "write" | "admin" {
  if (commandId.startsWith("settings.") || commandId.startsWith("rulesets.") || commandId.startsWith("pages.")) {
    return "admin";
  }

  if (
    commandId.includes(".create") ||
    commandId.includes(".edit") ||
    commandId.includes(".close") ||
    commandId.includes(".merge") ||
    commandId.includes(".reopen") ||
    commandId.includes(".review") ||
    commandId.includes(".delete") ||
    commandId.includes(".upload") ||
    commandId.includes(".set") ||
    commandId.includes(".add") ||
    commandId.includes(".remove") ||
    commandId.includes(".cancel") ||
    commandId.includes(".rerun")
  ) {
    return "write";
  }

  return "viewer";
}

function stringValue(value: unknown, fallback: string): string {
  if (typeof value === "string" && value.trim().length > 0) {
    return value;
  }

  return fallback;
}

function numberValue(value: unknown, fallback: number): number {
  if (typeof value === "number" && Number.isFinite(value)) {
    return value;
  }

  if (typeof value === "string") {
    const parsed = Number(value);
    if (Number.isFinite(parsed)) {
      return parsed;
    }
  }

  return fallback;
}

interface MockIssueComment {
  id: number;
  body: string;
  createdAt: string;
  author: { login: string };
  url: string;
}

interface MockIssueRecord {
  number: number;
  title: string;
  state: "OPEN" | "CLOSED";
  url: string;
  body: string;
  author: { login: string };
  labels: Array<{ name: string }>;
  assignees: Array<{ login: string }>;
  updatedAt: string;
  comments: MockIssueComment[];
}

type MockIssueStore = Record<string, MockIssueRecord[]>;

function createInitialMockState(): MockIssueStore {
  return {};
}

function ensureMockIssues(payload: Record<string, unknown>): MockIssueRecord[] {
  const owner = stringValue(payload.owner, "mock-user");
  const repo = stringValue(payload.repo, "gh-client");
  const key = `${owner}/${repo}`;
  if (!mockState[key]) {
    mockState[key] = createDefaultIssues(owner, repo);
  }
  return mockState[key];
}

function createDefaultIssues(owner: string, repo: string): MockIssueRecord[] {
  return [
    {
      number: 1,
      title: "Mock issue",
      state: "OPEN",
      url: `https://github.com/${owner}/${repo}/issues/1`,
      body: "This is a mock issue body. Use quick actions to test edits and comments.",
      author: { login: "reporter-a" },
      labels: [{ name: "bug" }, { name: "triage" }],
      assignees: [{ login: "mock-user" }],
      updatedAt: mockNow,
      comments: [
        {
          id: 1001,
          body: "Initial discussion comment.",
          createdAt: mockNow,
          author: { login: "maintainer-a" },
          url: `https://github.com/${owner}/${repo}/issues/1#issuecomment-1001`,
        },
      ],
    },
    {
      number: 2,
      title: "Closed issue sample",
      state: "CLOSED",
      url: `https://github.com/${owner}/${repo}/issues/2`,
      body: "Closed issue body.",
      author: { login: "reporter-b" },
      labels: [{ name: "done" }],
      assignees: [],
      updatedAt: mockNow,
      comments: [],
    },
  ];
}

function applyMockIssueEdit(issue: MockIssueRecord, payload: Record<string, unknown>): void {
  const addAssignees = stringArrayValue(payload.add_assignees);
  const removeAssignees = stringArrayValue(payload.remove_assignees);
  const addLabels = stringArrayValue(payload.add_labels);
  const removeLabels = stringArrayValue(payload.remove_labels);

  if (addAssignees.length > 0) {
    const existing = new Set(issue.assignees.map((entry) => entry.login.toLowerCase()));
    for (const assignee of addAssignees) {
      if (!existing.has(assignee.toLowerCase())) {
        issue.assignees.push({ login: assignee });
        existing.add(assignee.toLowerCase());
      }
    }
  }

  if (removeAssignees.length > 0) {
    const removes = new Set(removeAssignees.map((entry) => entry.toLowerCase()));
    issue.assignees = issue.assignees.filter(
      (entry) => !removes.has(entry.login.toLowerCase()),
    );
  }

  if (addLabels.length > 0) {
    const existing = new Set(issue.labels.map((entry) => entry.name.toLowerCase()));
    for (const label of addLabels) {
      if (!existing.has(label.toLowerCase())) {
        issue.labels.push({ name: label });
        existing.add(label.toLowerCase());
      }
    }
  }

  if (removeLabels.length > 0) {
    const removes = new Set(removeLabels.map((entry) => entry.toLowerCase()));
    issue.labels = issue.labels.filter((entry) => !removes.has(entry.name.toLowerCase()));
  }

  const title = stringValue(payload.title, "").trim();
  if (title.length > 0) {
    issue.title = title;
  }

  const body = stringValue(payload.body, "").trim();
  if (body.length > 0) {
    issue.body = body;
  }
}

function stringArrayValue(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }

  const deduped: string[] = [];
  for (const entry of value) {
    if (typeof entry !== "string") {
      continue;
    }
    const normalized = entry.trim();
    if (normalized.length === 0 || deduped.includes(normalized)) {
      continue;
    }
    deduped.push(normalized);
  }

  return deduped;
}
