import { COMMAND_CATALOG } from "./commandCatalog";
import { STABLE_COMMAND_IDS, type CommandId } from "./commandIds";

export function listCommandsForRoute(route: string): CommandId[] {
  if (route !== "settings") {
    return [];
  }

  return STABLE_COMMAND_IDS.filter((id) => id.startsWith("settings."));
}

export function commandOptions(): Array<{ value: CommandId; label: string }> {
  return STABLE_COMMAND_IDS.map((id) => ({ value: id, label: COMMAND_CATALOG[id].title }));
}
