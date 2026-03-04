import { COMMAND_CATALOG } from "./commandCatalog";
import { STABLE_COMMAND_IDS, type CommandId } from "./commandIds";

export function listCommandsForRoute(route: string): CommandId[] {
  switch (route) {
    case "dashboard":
      return ["auth.status", "repo.list"];
    case "repositories":
      return STABLE_COMMAND_IDS.filter((id) => id.startsWith("repo.") && id !== "repo.list");
    case "pull_requests":
      return STABLE_COMMAND_IDS.filter((id) => id.startsWith("pr."));
    case "issues":
      return STABLE_COMMAND_IDS.filter((id) => id.startsWith("issue."));
    case "actions":
      return STABLE_COMMAND_IDS.filter((id) => id.startsWith("workflow.") || id.startsWith("run."));
    case "releases":
      return STABLE_COMMAND_IDS.filter((id) => id.startsWith("release."));
    case "settings":
      return STABLE_COMMAND_IDS.filter((id) => id.startsWith("settings."));
    case "p2":
      return STABLE_COMMAND_IDS.filter(
        (id) =>
          id.startsWith("projects.") ||
          id.startsWith("discussions.") ||
          id.startsWith("wiki.") ||
          id.startsWith("pages.") ||
          id.startsWith("rulesets.") ||
          id.startsWith("insights."),
      );
    case "console":
      return [...STABLE_COMMAND_IDS];
    default:
      return [];
  }
}

export function commandOptions(): Array<{ value: CommandId; label: string }> {
  return STABLE_COMMAND_IDS.map((id) => ({ value: id, label: COMMAND_CATALOG[id].title }));
}
