import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Navigate, Route, Routes } from "react-router-dom";

import { JsonDrawer } from "./components/JsonDrawer";
import { Shell } from "./components/Shell";
import { executeCommand } from "./core/executor";
import { appendHistory } from "./core/history";
import { useI18n } from "./core/i18n";
import { normalizeViewerPermission } from "./core/permissions";
import type { CommandPermission, CommandSelectionOptions } from "./core/types";
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

interface RepoOption {
  owner: string;
  repo: string;
  viewerPermission: CommandPermission | null;
}

function App(): JSX.Element {
  const { t } = useI18n();
  const [owner, setOwner] = useState("");
  const [repo, setRepo] = useState("");
  const [repoPermission, setRepoPermission] = useState<CommandPermission | null>(null);
  const [authLoggedIn, setAuthLoggedIn] = useState(false);
  const [inspectState, setInspectState] = useState<InspectState>(initialInspect);
  const [ownerOptions, setOwnerOptions] = useState<string[]>([]);
  const [reposByOwner, setReposByOwner] = useState<Record<string, RepoOption[]>>({});
  const [contextLoading, setContextLoading] = useState(false);
  const [contextError, setContextError] = useState<string | null>(null);
  const [branchOptions, setBranchOptions] = useState<string[]>([]);
  const [pullRequestNumberOptions, setPullRequestNumberOptions] = useState<number[]>([]);
  const [issueNumberOptions, setIssueNumberOptions] = useState<number[]>([]);
  const [runIdOptions, setRunIdOptions] = useState<number[]>([]);
  const [releaseTagOptions, setReleaseTagOptions] = useState<string[]>([]);
  const contextRefreshSeq = useRef(0);
  const repoDataRefreshSeq = useRef(0);

  const repoLabel = useMemo(() => repoPermission ?? "unknown", [repoPermission]);
  const currentRepoOptions = useMemo(
    () => reposByOwner[owner] ?? [],
    [owner, reposByOwner],
  );
  const selectionOptions = useMemo<CommandSelectionOptions>(
    () => ({
      ownerOptions,
      repoOptions: currentRepoOptions.map((item) => item.repo),
      branchOptions,
      pullRequestNumberOptions,
      issueNumberOptions,
      runIdOptions,
      releaseTagOptions,
    }),
    [
      branchOptions,
      currentRepoOptions,
      issueNumberOptions,
      ownerOptions,
      pullRequestNumberOptions,
      releaseTagOptions,
      runIdOptions,
    ],
  );

  const refreshRepoDerivedOptions = useCallback(
    async (nextOwner: string, nextRepo: string) => {
      const requestSeq = ++repoDataRefreshSeq.current;

      if (!nextOwner || !nextRepo) {
        setBranchOptions([]);
        setPullRequestNumberOptions([]);
        setIssueNumberOptions([]);
        setRunIdOptions([]);
        setReleaseTagOptions([]);
        return;
      }

      const results = await Promise.allSettled([
        executeCommand("repo.branches.list", { owner: nextOwner, repo: nextRepo, limit: 100 }, { permission: "viewer" }),
        executeCommand("pr.list", { owner: nextOwner, repo: nextRepo, limit: 100 }, { permission: "viewer" }),
        executeCommand("issue.list", { owner: nextOwner, repo: nextRepo, limit: 100 }, { permission: "viewer" }),
        executeCommand("run.list", { owner: nextOwner, repo: nextRepo, limit: 100 }, { permission: "viewer" }),
        executeCommand("release.list", { owner: nextOwner, repo: nextRepo, limit: 100 }, { permission: "viewer" }),
      ]);

      if (requestSeq !== repoDataRefreshSeq.current) {
        return;
      }

      setBranchOptions(
        results[0].status === "fulfilled"
          ? extractStringList(results[0].value.data, ["name"])
          : [],
      );
      setPullRequestNumberOptions(
        results[1].status === "fulfilled"
          ? extractNumberList(results[1].value.data, ["number"])
          : [],
      );
      setIssueNumberOptions(
        results[2].status === "fulfilled"
          ? extractNumberList(results[2].value.data, ["number"])
          : [],
      );
      setRunIdOptions(
        results[3].status === "fulfilled"
          ? extractNumberList(results[3].value.data, ["databaseId", "database_id", "id"])
          : [],
      );
      setReleaseTagOptions(
        results[4].status === "fulfilled"
          ? extractStringList(results[4].value.data, ["tagName", "tag_name"])
          : [],
      );
    },
    [],
  );

  const refreshContextOptions = useCallback(async () => {
    const requestSeq = ++contextRefreshSeq.current;
    setContextLoading(true);
    setContextError(null);

    try {
      const auth = await executeCommand<{
        logged_in?: boolean;
        account?: string | null;
      }>("auth.status", {}, { permission: "viewer" });

      const loggedIn = Boolean(auth.data.logged_in);
      setAuthLoggedIn(loggedIn);

      if (!loggedIn) {
        if (requestSeq === contextRefreshSeq.current) {
          setOwnerOptions([]);
          setReposByOwner({});
          setOwner("");
          setRepo("");
          setRepoPermission(null);
          await refreshRepoDerivedOptions("", "");
        }
        return;
      }

      const accountOwner =
        typeof auth.data.account === "string" ? auth.data.account.trim() : "";

      const orgsResponse = await executeCommand<Array<{ login?: string }>>(
        "auth.organizations.list",
        {},
        { permission: "viewer" },
      );
      const orgOwners = orgsResponse.data
        .map((row) => (typeof row?.login === "string" ? row.login.trim() : ""))
        .filter((value) => value.length > 0);

      const owners = dedupeStrings([accountOwner, ...orgOwners], {
        preferredFirst: accountOwner || undefined,
      });

      const repoEntries = await Promise.all(
        owners.map(async (ownerName) => {
          const list = await executeCommand<Array<Record<string, unknown>>>(
            "repo.list",
            { owner: ownerName, limit: 100 },
            { permission: "viewer" },
          );

          return [ownerName, parseRepoOptions(ownerName, list.data)] as const;
        }),
      );

      if (requestSeq !== contextRefreshSeq.current) {
        return;
      }

      const nextReposByOwner = Object.fromEntries(repoEntries);
      setOwnerOptions(owners);
      setReposByOwner(nextReposByOwner);

      const nextOwner =
        owner && owners.includes(owner) ? owner : owners[0] ?? "";
      const options = nextReposByOwner[nextOwner] ?? [];
      const nextRepo =
        repo && options.some((item) => item.repo === repo)
          ? repo
          : options[0]?.repo ?? "";
      const selected = options.find((item) => item.repo === nextRepo);

      setOwner(nextOwner);
      setRepo(nextRepo);
      setRepoPermission(selected?.viewerPermission ?? null);

      await refreshRepoDerivedOptions(nextOwner, nextRepo);
    } catch (error) {
      if (requestSeq !== contextRefreshSeq.current) {
        return;
      }

      const message = error instanceof Error ? error.message : String(error);
      setContextError(message);
    } finally {
      if (requestSeq === contextRefreshSeq.current) {
        setContextLoading(false);
      }
    }
  }, [owner, refreshRepoDerivedOptions, repo]);

  useEffect(() => {
    void refreshContextOptions();
  }, [refreshContextOptions]);

  useEffect(() => {
    void refreshRepoDerivedOptions(owner, repo);
  }, [owner, repo, refreshRepoDerivedOptions]);

  const handleOwnerChange = useCallback(
    (nextOwner: string) => {
      setOwner(nextOwner);
      const options = reposByOwner[nextOwner] ?? [];
      const nextRepo = options.some((item) => item.repo === repo)
        ? repo
        : options[0]?.repo ?? "";
      const selected = options.find((item) => item.repo === nextRepo);
      setRepo(nextRepo);
      setRepoPermission(selected?.viewerPermission ?? null);
    },
    [repo, reposByOwner],
  );

  const handleRepoChange = useCallback(
    (nextRepo: string) => {
      setRepo(nextRepo);
      const selected = (reposByOwner[owner] ?? []).find(
        (item) => item.repo === nextRepo,
      );
      setRepoPermission(selected?.viewerPermission ?? null);
    },
    [owner, reposByOwner],
  );

  const handleExecuted = useCallback(
    (event: CommandExecutionEvent) => {
      if (event.commandId === "auth.status" && event.status === "success") {
        const data = event.data as { logged_in?: boolean } | undefined;
        setAuthLoggedIn(Boolean(data?.logged_in));
        void refreshContextOptions();
      }

      if (event.status === "error" && event.error?.code === "auth_required") {
        setAuthLoggedIn(false);
        void refreshContextOptions();
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
    [owner, refreshContextOptions, repo],
  );

  return (
    <>
      <Shell
        owner={owner}
        repo={repo}
        ownerOptions={ownerOptions}
        repoOptions={currentRepoOptions.map((item) => item.repo)}
        contextLoading={contextLoading}
        contextError={contextError}
        onRefreshContext={() => {
          void refreshContextOptions();
        }}
        onOwnerChange={handleOwnerChange}
        onRepoChange={handleRepoChange}
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
                selectionOptions={selectionOptions}
                onExecuted={handleExecuted}
                onInspect={(title, value) => setInspectState({ open: true, title, value })}
                onAuthStateChange={setAuthLoggedIn}
                onRepoContextChange={(nextOwner, nextRepo, permission) => {
                  handleOwnerChange(nextOwner);
                  setRepo(nextRepo);
                  if (permission) {
                    setRepoPermission((permission as CommandPermission | undefined) ?? null);
                    return;
                  }

                  const selected = (reposByOwner[nextOwner] ?? []).find(
                    (item) => item.repo === nextRepo,
                  );
                  setRepoPermission(selected?.viewerPermission ?? null);
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
                selectionOptions={selectionOptions}
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
                selectionOptions={selectionOptions}
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
                selectionOptions={selectionOptions}
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
                selectionOptions={selectionOptions}
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
                selectionOptions={selectionOptions}
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
                selectionOptions={selectionOptions}
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
                selectionOptions={selectionOptions}
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
                selectionOptions={selectionOptions}
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

function parseRepoOptions(
  owner: string,
  rows: Array<Record<string, unknown>>,
): RepoOption[] {
  const options: RepoOption[] = [];
  const seen = new Set<string>();

  for (const row of rows) {
    const record = asRecord(row);
    const nameWithOwner = asString(record.nameWithOwner);
    const repoFromNameWithOwner =
      nameWithOwner && nameWithOwner.includes("/")
        ? nameWithOwner.split("/")[1]
        : null;
    const repoName = repoFromNameWithOwner ?? asString(record.name);

    if (!repoName) {
      continue;
    }

    if (seen.has(repoName)) {
      continue;
    }
    seen.add(repoName);

    options.push({
      owner,
      repo: repoName,
      viewerPermission: normalizeViewerPermission(
        asString(record.viewerPermission) ?? undefined,
      ),
    });
  }

  return options;
}

function dedupeStrings(
  values: string[],
  options: { preferredFirst?: string } = {},
): string[] {
  const normalized = values
    .map((value) => value.trim())
    .filter((value) => value.length > 0);

  const unique: string[] = [];
  for (const value of normalized) {
    if (!unique.includes(value)) {
      unique.push(value);
    }
  }

  if (options.preferredFirst && unique.includes(options.preferredFirst)) {
    return [
      options.preferredFirst,
      ...unique.filter((value) => value !== options.preferredFirst),
    ];
  }

  return unique;
}

function extractStringList(data: unknown, keys: string[]): string[] {
  if (!Array.isArray(data)) {
    return [];
  }

  const values: string[] = [];
  for (const row of data) {
    const record = asRecord(row);
    for (const key of keys) {
      const value = asString(record[key]);
      if (value) {
        values.push(value);
        break;
      }
    }
  }

  return dedupeStrings(values);
}

function extractNumberList(data: unknown, keys: string[]): number[] {
  if (!Array.isArray(data)) {
    return [];
  }

  const values: number[] = [];
  for (const row of data) {
    const record = asRecord(row);
    for (const key of keys) {
      const raw = record[key];
      const value =
        typeof raw === "number"
          ? raw
          : typeof raw === "string"
            ? Number(raw)
            : NaN;
      if (Number.isFinite(value) && value > 0) {
        values.push(value);
        break;
      }
    }
  }

  const unique = Array.from(new Set(values));
  unique.sort((left, right) => right - left);
  return unique;
}

function asRecord(value: unknown): Record<string, unknown> {
  if (typeof value === "object" && value !== null) {
    return value as Record<string, unknown>;
  }
  return {};
}

function asString(value: unknown): string | null {
  if (typeof value === "string" && value.trim().length > 0) {
    return value.trim();
  }
  return null;
}

export default App;
