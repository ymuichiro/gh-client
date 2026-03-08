import type { CommandEnvelope } from "./types";
import type { CommandId } from "./commandIds";

const mockNow = new Date().toISOString();

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
      return [
        {
          number: 1,
          title: "Mock issue",
          state: "OPEN",
          url: "https://github.com/mock-user/gh-client/issues/1",
          author: { login: "reporter-a" },
          labels: [{ name: "bug" }, { name: "triage" }],
          assignees: [{ login: "mock-user" }],
          updatedAt: mockNow,
        },
        {
          number: 2,
          title: "Closed issue sample",
          state: "CLOSED",
          url: "https://github.com/mock-user/gh-client/issues/2",
          author: { login: "reporter-b" },
          labels: [{ name: "done" }],
          assignees: [],
          updatedAt: mockNow,
        },
      ];

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
