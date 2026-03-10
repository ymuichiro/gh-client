import { CommandBoard } from "../components/CommandBoard";
import { listCommandsForRoute } from "../core/pageCommands";
import type { PageSharedProps } from "./types";

interface FeaturePageProps extends PageSharedProps {
  route: "settings";
  title: string;
  description?: string;
}

export function FeaturePage({ route, title, description, ...rest }: FeaturePageProps): JSX.Element {
  const commands = listCommandsForRoute(route);

  return (
    <CommandBoard
      title={title}
      description={description}
      commandIds={commands}
      owner={rest.owner}
      repo={rest.repo}
      repoPermission={rest.repoPermission}
      selectionOptions={rest.selectionOptions}
      onExecuted={rest.onExecuted}
      onInspect={rest.onInspect}
    />
  );
}
