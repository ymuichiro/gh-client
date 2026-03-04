import type { CommandEnvelope } from "./types";
import type { CommandId } from "./commandIds";

const mockNow = new Date().toISOString();

export async function executeMockEnvelope(envelope: CommandEnvelope): Promise<unknown> {
  switch (envelope.command_id) {
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
        },
      ];

    case "issue.list":
      return [
        {
          number: 1,
          title: "Mock issue",
          state: "OPEN",
          url: "https://github.com/mock-user/gh-client/issues/1",
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

    case "pr.diff.files.list":
      return [
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
