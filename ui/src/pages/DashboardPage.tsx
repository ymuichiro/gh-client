import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";

import { CommandBoard } from "../components/CommandBoard";
import { executeCommand } from "../core/executor";
import { normalizeViewerPermission } from "../core/permissions";
import { useI18n } from "../core/i18n";
import type { PageSharedProps } from "./types";

interface DashboardPageProps extends PageSharedProps {
  onAuthStateChange: (loggedIn: boolean) => void;
  onRepoContextChange: (owner: string, repo: string, permission: string | undefined) => void;
}

export function DashboardPage({
  owner,
  repo,
  repoPermission,
  onExecuted,
  onInspect,
  onAuthStateChange,
  onRepoContextChange,
}: DashboardPageProps): JSX.Element {
  const { t } = useI18n();

  const authQuery = useQuery({
    queryKey: ["auth-status"],
    queryFn: async () => {
      const result = await executeCommand("auth.status", {}, { permission: "viewer" });
      const loggedIn = Boolean((result.data as { logged_in?: boolean }).logged_in);
      onAuthStateChange(loggedIn);
      return result.data as { logged_in?: boolean; account?: string | null; host?: string | null };
    },
    staleTime: 30_000,
    retry: 1,
  });

  const reposQuery = useQuery({
    queryKey: ["repo-list", owner],
    queryFn: async () => {
      const result = await executeCommand("repo.list", { owner, limit: 30 }, { permission: "viewer" });
      return result.data as Array<{
        name: string;
        nameWithOwner: string;
        description?: string | null;
        viewerPermission?: string;
      }>;
    },
    enabled: owner.trim().length > 0,
    staleTime: 30_000,
    retry: 1,
  });

  const authLabel = useMemo(() => {
    if (authQuery.isLoading) {
      return t("common.loading");
    }

    if (authQuery.isError || !authQuery.data) {
      return t("auth.logged_out");
    }

    return authQuery.data.logged_in ? t("auth.logged_in") : t("auth.logged_out");
  }, [authQuery.data, authQuery.isError, authQuery.isLoading, t]);

  return (
    <div className="stack-lg">
      <section className="hero-card">
        <h2>{t("nav.dashboard")}</h2>
        <p>{t("dashboard.description")}</p>
        <p>
          {t("header.auth")}: <strong>{authLabel}</strong>
        </p>
        <p>{t("common.gh_login_hint")}</p>
        <div className="row gap-sm">
          <button type="button" className="btn" onClick={() => authQuery.refetch()}>
            auth.status
          </button>
          <button type="button" className="btn secondary" onClick={() => reposQuery.refetch()}>
            repo.list
          </button>
        </div>
      </section>

      <section className="table-section">
        <header className="section-header">
          <h3>Repositories</h3>
          <p>repo.list response</p>
        </header>
        {reposQuery.isLoading ? <p>{t("common.loading")}</p> : null}
        {reposQuery.error ? <p className="error-text">{String(reposQuery.error)}</p> : null}
        {reposQuery.data?.length ? (
          <table className="repo-table">
            <thead>
              <tr>
                <th>Name</th>
                <th>Permission</th>
                <th>Description</th>
                <th>Action</th>
              </tr>
            </thead>
            <tbody>
              {reposQuery.data.map((repoRow) => {
                const [rowOwner, rowRepo] = repoRow.nameWithOwner.split("/");
                const normalized = normalizeViewerPermission(repoRow.viewerPermission);
                return (
                  <tr key={repoRow.nameWithOwner}>
                    <td>{repoRow.nameWithOwner}</td>
                    <td>{repoRow.viewerPermission ?? "UNKNOWN"}</td>
                    <td>{repoRow.description ?? ""}</td>
                    <td>
                      <button
                        type="button"
                        className="btn secondary"
                        onClick={() => onRepoContextChange(rowOwner ?? owner, rowRepo ?? repo, normalized ?? undefined)}
                      >
                        Select
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        ) : (
          <p>No repositories</p>
        )}
      </section>

      <CommandBoard
        title="Dashboard Commands"
        commandIds={["auth.status", "repo.list"]}
        owner={owner}
        repo={repo}
        repoPermission={repoPermission}
        onExecuted={onExecuted}
        onInspect={onInspect}
      />
    </div>
  );
}
