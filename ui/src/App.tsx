import { useCallback, useMemo, useState } from "react";
import { Navigate, Route, Routes } from "react-router-dom";

import type { CommandExecutionEvent } from "./components/CommandForm";
import { JsonDrawer } from "./components/JsonDrawer";
import { Shell } from "./components/Shell";
import { appendHistory } from "./core/history";
import { useI18n } from "./core/i18n";
import {
  groupRepositoriesByOwner,
  loadRepositoryScope,
  type RepositoryScopeConfig,
} from "./core/repositoryScope";
import { InboxPage } from "./pages/InboxPage";
import { RepositorySelectionPage } from "./pages/RepositorySelectionPage";

interface InspectState {
  open: boolean;
  title: string;
  value: unknown;
}

const initialInspect: InspectState = {
  open: false,
  title: "",
  value: null,
};

function App(): JSX.Element {
  const { t } = useI18n();
  const [scopeConfig, setScopeConfig] = useState<RepositoryScopeConfig | null>(() =>
    loadRepositoryScope(),
  );
  const [inspectState, setInspectState] = useState<InspectState>(initialInspect);

  const reposByOwner = useMemo(() => {
    if (!scopeConfig) {
      return {};
    }

    return groupRepositoriesByOwner(scopeConfig.repositories);
  }, [scopeConfig]);

  const allRepoTargets = useMemo(
    () =>
      Object.entries(reposByOwner).flatMap(([ownerName, options]) =>
        options.map((option) => ({
          owner: ownerName,
          repo: option.repo,
          viewerPermission: option.viewerPermission,
        })),
      ),
    [reposByOwner],
  );

  const hasRepositoryScope = allRepoTargets.length > 0;

  const handleApplyScope = useCallback(
    (config: RepositoryScopeConfig) => {
      setScopeConfig(config);
    },
    [],
  );

  const handleExecuted = useCallback(
    (event: CommandExecutionEvent) => {
      appendHistory({
        timestamp: new Date().toISOString(),
        requestId: event.requestId,
        commandId: event.commandId,
        status: event.status,
        code: event.error?.code,
      });
    },
    [],
  );

  return (
    <>
      <Shell>
        <Routes>
          <Route
            path="/"
            element={
              <Navigate to={hasRepositoryScope ? "/issues" : "/settings"} replace />
            }
          />
          <Route path="/inbox" element={<Navigate to="/issues" replace />} />
          <Route
            path="/repositories/select"
            element={<Navigate to="/settings" replace />}
          />
          <Route
            path="/issues"
            element={
              hasRepositoryScope ? (
                <InboxPage
                  repoTargets={allRepoTargets}
                  onExecuted={handleExecuted}
                  onInspect={(title, value) => setInspectState({ open: true, title, value })}
                  mode="issue"
                  title={t("nav.issues")}
                  subtitle={t("issues.subtitle")}
                />
              ) : (
                <Navigate to="/settings" replace />
              )
            }
          />
          <Route
            path="/pull-requests"
            element={
              hasRepositoryScope ? (
                <InboxPage
                  repoTargets={allRepoTargets}
                  onExecuted={handleExecuted}
                  onInspect={(title, value) => setInspectState({ open: true, title, value })}
                  mode="pr"
                  title={t("nav.pull_requests")}
                  subtitle={t("pull_requests.subtitle")}
                />
              ) : (
                <Navigate to="/settings" replace />
              )
            }
          />
          <Route
            path="/settings"
            element={
              <RepositorySelectionPage
                initialConfig={scopeConfig}
                onApplyConfig={handleApplyScope}
              />
            }
          />
          <Route
            path="*"
            element={<Navigate to={hasRepositoryScope ? "/issues" : "/settings"} replace />}
          />
        </Routes>
      </Shell>

      <JsonDrawer
        open={inspectState.open}
        title={inspectState.title}
        value={inspectState.value}
        onClose={() => setInspectState(initialInspect)}
      />
    </>
  );
}

export default App;
