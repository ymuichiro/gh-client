import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Navigate, Route, Routes, useLocation } from "react-router-dom";

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
  const location = useLocation();
  const [scopeConfig, setScopeConfig] = useState<RepositoryScopeConfig | null>(() =>
    loadRepositoryScope(),
  );
  const [inspectState, setInspectState] = useState<InspectState>(initialInspect);
  const [pendingInboxNavigationToken, setPendingInboxNavigationToken] = useState<number | null>(
    null,
  );

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

  const routeTransitionKey = `${location.pathname}${location.search}`;
  const [routeTransitionPhase, setRouteTransitionPhase] = useState<"pre" | "active">("active");
  const [routeProgressActive, setRouteProgressActive] = useState(false);
  const routeTransitionFrame = useRef<number | null>(null);
  const routeProgressTimer = useRef<number | null>(null);
  const navigationTokenRef = useRef(0);

  const isInboxPath = useCallback((path: string): boolean => {
    return path.startsWith("/issues") || path.startsWith("/pull-requests");
  }, []);

  const triggerRouteFeedback = useCallback((durationMs = 520) => {
    if (routeTransitionFrame.current !== null) {
      window.cancelAnimationFrame(routeTransitionFrame.current);
      routeTransitionFrame.current = null;
    }
    if (routeProgressTimer.current !== null) {
      window.clearTimeout(routeProgressTimer.current);
      routeProgressTimer.current = null;
    }

    setRouteTransitionPhase("pre");
    setRouteProgressActive(true);
    routeTransitionFrame.current = window.requestAnimationFrame(() => {
      setRouteTransitionPhase("active");
      routeTransitionFrame.current = null;
    });
    routeProgressTimer.current = window.setTimeout(() => {
      setRouteProgressActive(false);
      routeProgressTimer.current = null;
    }, durationMs);
  }, []);

  useEffect(() => {
    triggerRouteFeedback(620);
  }, [routeTransitionKey, triggerRouteFeedback]);

  useEffect(() => {
    return () => {
      if (routeTransitionFrame.current !== null) {
        window.cancelAnimationFrame(routeTransitionFrame.current);
      }
      if (routeProgressTimer.current !== null) {
        window.clearTimeout(routeProgressTimer.current);
      }
    };
  }, []);

  const handleNavigateStart = useCallback(
    (to: string) => {
      if (to === location.pathname) {
        return;
      }

      triggerRouteFeedback(620);
      if (isInboxPath(to)) {
        navigationTokenRef.current += 1;
        setPendingInboxNavigationToken(navigationTokenRef.current);
        return;
      }
      setPendingInboxNavigationToken(null);
    },
    [isInboxPath, location.pathname, triggerRouteFeedback],
  );

  return (
    <>
      <Shell
        onNavigateStart={handleNavigateStart}
      >
        <div
          className={routeProgressActive ? "route-progress active" : "route-progress"}
          aria-hidden="true"
        />
        <div
          className={
            routeTransitionPhase === "pre"
              ? "route-transition route-transition-pre"
              : "route-transition route-transition-active"
          }
          aria-live="polite"
        >
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
                    navigationLoadingToken={pendingInboxNavigationToken ?? undefined}
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
                    navigationLoadingToken={pendingInboxNavigationToken ?? undefined}
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
        </div>
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
