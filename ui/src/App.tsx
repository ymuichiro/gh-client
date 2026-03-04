import { useCallback, useMemo, useState } from "react";
import { Navigate, Route, Routes } from "react-router-dom";

import { JsonDrawer } from "./components/JsonDrawer";
import { Shell } from "./components/Shell";
import { appendHistory } from "./core/history";
import { useI18n } from "./core/i18n";
import type { CommandPermission } from "./core/types";
import { DashboardPage } from "./pages/DashboardPage";
import { FeaturePage } from "./pages/FeaturePage";
import { HistoryPage } from "./pages/HistoryPage";
import type { CommandExecutionEvent } from "./components/CommandForm";

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
  const [owner, setOwner] = useState("");
  const [repo, setRepo] = useState("");
  const [repoPermission, setRepoPermission] = useState<CommandPermission | null>(null);
  const [authLoggedIn, setAuthLoggedIn] = useState(false);
  const [inspectState, setInspectState] = useState<InspectState>(initialInspect);

  const repoLabel = useMemo(() => repoPermission ?? "unknown", [repoPermission]);

  const handleExecuted = useCallback(
    (event: CommandExecutionEvent) => {
      if (event.commandId === "auth.status" && event.status === "success") {
        const data = event.data as { logged_in?: boolean } | undefined;
        setAuthLoggedIn(Boolean(data?.logged_in));
      }

      if (event.status === "error" && event.error?.code === "auth_required") {
        setAuthLoggedIn(false);
      }

      appendHistory({
        timestamp: new Date().toISOString(),
        requestId: event.requestId,
        commandId: event.commandId,
        repo: owner && repo ? `${owner}/${repo}` : undefined,
        status: event.status,
        code: event.error?.code,
      });
    },
    [owner, repo],
  );

  return (
    <>
      <Shell
        owner={owner}
        repo={repo}
        onOwnerChange={setOwner}
        onRepoChange={setRepo}
        onRepoPermissionChange={(value) =>
          setRepoPermission(value === "unknown" ? null : value)
        }
        repoPermission={repoLabel}
        authLoggedIn={authLoggedIn}
      >
        <Routes>
          <Route
            path="/"
            element={
              <DashboardPage
                owner={owner}
                repo={repo}
                repoPermission={repoPermission}
                onExecuted={handleExecuted}
                onInspect={(title, value) => setInspectState({ open: true, title, value })}
                onAuthStateChange={setAuthLoggedIn}
                onRepoContextChange={(nextOwner, nextRepo, permission) => {
                  setOwner(nextOwner);
                  setRepo(nextRepo);
                  setRepoPermission((permission as CommandPermission | undefined) ?? null);
                }}
              />
            }
          />
          <Route
            path="/repositories"
            element={
              <FeaturePage
                route="repositories"
                title={t("nav.repositories")}
                owner={owner}
                repo={repo}
                repoPermission={repoPermission}
                onExecuted={handleExecuted}
                onInspect={(title, value) => setInspectState({ open: true, title, value })}
              />
            }
          />
          <Route
            path="/pull-requests"
            element={
              <FeaturePage
                route="pull_requests"
                title={t("nav.pull_requests")}
                owner={owner}
                repo={repo}
                repoPermission={repoPermission}
                onExecuted={handleExecuted}
                onInspect={(title, value) => setInspectState({ open: true, title, value })}
              />
            }
          />
          <Route
            path="/issues"
            element={
              <FeaturePage
                route="issues"
                title={t("nav.issues")}
                owner={owner}
                repo={repo}
                repoPermission={repoPermission}
                onExecuted={handleExecuted}
                onInspect={(title, value) => setInspectState({ open: true, title, value })}
              />
            }
          />
          <Route
            path="/actions"
            element={
              <FeaturePage
                route="actions"
                title={t("nav.actions")}
                owner={owner}
                repo={repo}
                repoPermission={repoPermission}
                onExecuted={handleExecuted}
                onInspect={(title, value) => setInspectState({ open: true, title, value })}
              />
            }
          />
          <Route
            path="/releases"
            element={
              <FeaturePage
                route="releases"
                title={t("nav.releases")}
                owner={owner}
                repo={repo}
                repoPermission={repoPermission}
                onExecuted={handleExecuted}
                onInspect={(title, value) => setInspectState({ open: true, title, value })}
              />
            }
          />
          <Route
            path="/settings"
            element={
              <FeaturePage
                route="settings"
                title={t("nav.settings")}
                owner={owner}
                repo={repo}
                repoPermission={repoPermission}
                onExecuted={handleExecuted}
                onInspect={(title, value) => setInspectState({ open: true, title, value })}
              />
            }
          />
          <Route
            path="/p2"
            element={
              <FeaturePage
                route="p2"
                title={t("nav.p2")}
                description={t("p2.description")}
                owner={owner}
                repo={repo}
                repoPermission={repoPermission}
                onExecuted={handleExecuted}
                onInspect={(title, value) => setInspectState({ open: true, title, value })}
              />
            }
          />
          <Route
            path="/console"
            element={
              <FeaturePage
                route="console"
                title={t("console.title")}
                description={t("console.description")}
                owner={owner}
                repo={repo}
                repoPermission={repoPermission}
                onExecuted={handleExecuted}
                onInspect={(title, value) => setInspectState({ open: true, title, value })}
              />
            }
          />
          <Route path="/history" element={<HistoryPage />} />
          <Route path="*" element={<Navigate to="/" replace />} />
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
