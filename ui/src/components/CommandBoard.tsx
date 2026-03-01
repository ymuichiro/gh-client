import { COMMAND_CATALOG } from "../core/commandCatalog";
import type { CommandId, } from "../core/commandIds";
import type { CommandPermission } from "../core/types";
import { CommandForm, type CommandExecutionEvent } from "./CommandForm";

interface CommandBoardProps {
  title: string;
  description?: string;
  commandIds: CommandId[];
  owner: string;
  repo: string;
  repoPermission: CommandPermission | null;
  onExecuted: (event: CommandExecutionEvent) => void;
  onInspect: (title: string, value: unknown) => void;
}

export function CommandBoard({
  title,
  description,
  commandIds,
  owner,
  repo,
  repoPermission,
  onExecuted,
  onInspect,
}: CommandBoardProps): JSX.Element {
  return (
    <section className="page-section">
      <header className="section-header">
        <h2>{title}</h2>
        {description ? <p>{description}</p> : null}
      </header>

      <div className="command-grid">
        {commandIds.map((commandId) => (
          <CommandForm
            key={commandId}
            spec={COMMAND_CATALOG[commandId]}
            owner={owner}
            repo={repo}
            repoPermission={repoPermission}
            onExecuted={onExecuted}
            onInspect={onInspect}
          />
        ))}
      </div>
    </section>
  );
}
