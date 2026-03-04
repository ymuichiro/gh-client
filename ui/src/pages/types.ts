import type { CommandPermission } from "../core/types";
import type { CommandExecutionEvent } from "../components/CommandForm";

export interface PageSharedProps {
  owner: string;
  repo: string;
  repoPermission: CommandPermission | null;
  onExecuted: (event: CommandExecutionEvent) => void;
  onInspect: (title: string, value: unknown) => void;
}
