import type { CommandPermission, CommandSelectionOptions } from "../core/types";
import type { CommandExecutionEvent } from "../components/CommandForm";

export interface PageSharedProps {
  owner: string;
  repo: string;
  repoPermission: CommandPermission | null;
  selectionOptions: CommandSelectionOptions;
  onExecuted: (event: CommandExecutionEvent) => void;
  onInspect: (title: string, value: unknown) => void;
}
