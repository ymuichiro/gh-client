import { useCallback, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";

import { executeCommand } from "../core/executor";
import { useI18n } from "../core/i18n";
import { normalizeViewerPermission } from "../core/permissions";
import {
  type RepositoryScopeConfig,
  type ScopedRepositoryTarget,
  dedupeRepositories,
  groupRepositoriesByOwner,
  saveRepositoryScope,
  toRepositoryKey,
} from "../core/repositoryScope";

interface RepositorySelectionPageProps {
  initialConfig: RepositoryScopeConfig | null;
  onApplyConfig: (config: RepositoryScopeConfig) => void;
}

export function RepositorySelectionPage({
  initialConfig,
  onApplyConfig,
}: RepositorySelectionPageProps): JSX.Element {
  const { t } = useI18n();
  const navigate = useNavigate();

  const [orgCandidates, setOrgCandidates] = useState<string[]>(() =>
    dedupeStrings(initialConfig?.orgs ?? []),
  );
  const [selectedOrgs, setSelectedOrgs] = useState<string[]>(() => initialConfig?.orgs ?? []);
  const [orgLoading, setOrgLoading] = useState(false);
  const [orgError, setOrgError] = useState<string | null>(null);

  const [repositoriesByOwner, setRepositoriesByOwner] = useState<
    Record<string, ScopedRepositoryTarget[]>
  >(() => groupRepositoriesByOwner(initialConfig?.repositories ?? []));
  const [selectedRepoKeys, setSelectedRepoKeys] = useState<string[]>(
    initialConfig?.repositories.map((repository) =>
      toRepositoryKey(repository.owner, repository.repo),
    ) ?? [],
  );
  const [repoLoading, setRepoLoading] = useState(false);
  const [repoError, setRepoError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");

  const fmt = useCallback(
    (key: string, vars: Record<string, string | number>) => formatTemplate(t(key), vars),
    [t],
  );

  const loadOrganizations = useCallback(async () => {
    setOrgLoading(true);
    setOrgError(null);

    try {
      const auth = await executeCommand<{
        logged_in?: boolean;
        account?: string | null;
      }>("auth.status", {}, { permission: "viewer" });

      const loggedIn = Boolean(auth.data.logged_in);

      if (!loggedIn) {
        setOrgCandidates([]);
        setSelectedOrgs([]);
        setOrgError(t("repo_selection.error.auth_required"));
        return;
      }

      const accountOwner = typeof auth.data.account === "string" ? auth.data.account.trim() : "";
      const organizations = await executeCommand<Array<{ login?: string }>>(
        "auth.organizations.list",
        {},
        { permission: "viewer" },
      );

      const candidates = dedupeStrings([
        accountOwner,
        ...organizations.data
          .map((entry) => (typeof entry?.login === "string" ? entry.login.trim() : ""))
          .filter((entry) => entry.length > 0),
      ]);

      setOrgCandidates(candidates);
      setSelectedOrgs((current) => current.filter((owner) => candidates.includes(owner)));
    } catch (cause) {
      setOrgError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setOrgLoading(false);
    }
  }, [t]);

  const updateRepositoryCandidates = useCallback(async () => {
    if (selectedOrgs.length === 0) {
      setRepoError(t("repo_selection.error.org_required"));
      return;
    }

    setRepoLoading(true);
    setRepoError(null);

    try {
      const listed = await Promise.all(
        selectedOrgs.map(async (owner) => {
          const response = await executeCommand<unknown[]>(
            "repo.list",
            { owner, limit: 100 },
            { permission: "viewer" },
          );

          return [owner, parseRepositories(owner, response.data)] as const;
        }),
      );

      const nextByOwner = Object.fromEntries(listed);
      const availableKeys = new Set(
        Object.values(nextByOwner)
          .flat()
          .map((repository) => toRepositoryKey(repository.owner, repository.repo)),
      );

      setRepositoriesByOwner(nextByOwner);
      setSelectedRepoKeys((current) => current.filter((key) => availableKeys.has(key)));
    } catch (cause) {
      setRepoError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setRepoLoading(false);
    }
  }, [selectedOrgs, t]);

  const visibleRepositoriesByOwner = useMemo(() => {
    const query = searchQuery.trim().toLowerCase();
    const byOwner: Record<string, ScopedRepositoryTarget[]> = {};

    for (const owner of selectedOrgs) {
      const repositories = repositoriesByOwner[owner] ?? [];
      const filtered =
        query.length === 0
          ? repositories.slice(0, 10)
          : repositories.filter((repository) => repository.repo.toLowerCase().includes(query));

      byOwner[owner] = filtered;
    }

    return byOwner;
  }, [repositoriesByOwner, searchQuery, selectedOrgs]);

  const selectedRepositoryCount = selectedRepoKeys.length;

  const confirmSelection = useCallback(() => {
    if (selectedOrgs.length === 0) {
      setRepoError(t("repo_selection.error.org_required"));
      return;
    }

    const selectedOrgSet = new Set(selectedOrgs);
    const selectedRepositories = dedupeRepositories(
      Object.values(repositoriesByOwner)
        .flat()
        .filter(
          (repository) =>
            selectedOrgSet.has(repository.owner) &&
            selectedRepoKeys.includes(toRepositoryKey(repository.owner, repository.repo)),
        ),
    );

    if (selectedRepositories.length === 0) {
      setRepoError(t("repo_selection.error.repo_required"));
      return;
    }

    const config: RepositoryScopeConfig = {
      orgs: [...selectedOrgs],
      repositories: selectedRepositories,
      updatedAt: new Date().toISOString(),
    };

    saveRepositoryScope(config);
    onApplyConfig(config);
    navigate("/issues", { replace: true });
  }, [navigate, onApplyConfig, repositoriesByOwner, selectedOrgs, selectedRepoKeys, t]);

  return (
    <section className="page-section">
      <header className="section-header">
        <h2>{t("repo_selection.title")}</h2>
        <p>{t("repo_selection.subtitle")}</p>
      </header>

      <section className="hero-card stack-lg">
        <div className="section-header">
          <h3>{t("repo_selection.org.title")}</h3>
          <p>{t("repo_selection.org.description")}</p>
        </div>

        <div className="row gap-sm wrap">
          <button
            type="button"
            className="btn secondary"
            onClick={() => void loadOrganizations()}
            disabled={orgLoading}
          >
            {orgLoading ? t("repo_selection.org.loading") : t("repo_selection.org.reload")}
          </button>
          <button
            type="button"
            className="btn"
            onClick={() => {
              void updateRepositoryCandidates();
            }}
            disabled={repoLoading || selectedOrgs.length === 0}
          >
            {repoLoading ? t("repo_selection.repo.loading") : t("repo_selection.repo.update")}
          </button>
        </div>

        {orgError ? <p className="error-text">{orgError}</p> : null}

        <div className="scope-grid">
          {orgCandidates.map((owner) => {
            const checked = selectedOrgs.includes(owner);
            return (
              <label key={owner} className="scope-option">
                <input
                  type="checkbox"
                  checked={checked}
                  onChange={(event) => {
                    setSelectedOrgs((current) => {
                      if (event.target.checked) {
                        if (current.includes(owner)) {
                          return current;
                        }
                        return [...current, owner];
                      }

                      return current.filter((value) => value !== owner);
                    });

                    if (!event.target.checked) {
                      setSelectedRepoKeys((current) =>
                        current.filter((key) => !key.startsWith(`${owner}/`)),
                      );
                    }
                  }}
                />
                <span>{owner}</span>
              </label>
            );
          })}

          {orgCandidates.length === 0 ? (
            <p className="info-text">{t("repo_selection.org.empty")}</p>
          ) : null}
        </div>
      </section>

      <section className="table-section stack-lg">
        <div className="section-header">
          <h3>{t("repo_selection.repo.title")}</h3>
          <p>{t("repo_selection.repo.description")}</p>
        </div>

        <label>
          <span>{t("repo_selection.repo.search")}</span>
          <input
            className="input"
            value={searchQuery}
            onChange={(event) => setSearchQuery(event.target.value)}
            placeholder={t("repo_selection.repo.search_placeholder")}
          />
        </label>

        {repoError ? <p className="error-text">{repoError}</p> : null}

        <div className="scope-groups">
          {selectedOrgs.map((owner) => {
            const visible = visibleRepositoriesByOwner[owner] ?? [];
            const total = repositoriesByOwner[owner]?.length ?? 0;

            return (
              <article key={owner} className="scope-group">
                <header>
                  <strong>{owner}</strong>
                  {searchQuery.trim().length === 0 && total > 10 ? (
                    <span className="info-text">{t("repo_selection.repo.top10_hint")}</span>
                  ) : null}
                </header>

                <div className="scope-grid">
                  {visible.map((repository) => {
                    const key = toRepositoryKey(repository.owner, repository.repo);
                    return (
                      <label key={key} className="scope-option">
                        <input
                          type="checkbox"
                          checked={selectedRepoKeys.includes(key)}
                          onChange={(event) => {
                            setSelectedRepoKeys((current) => {
                              if (event.target.checked) {
                                if (current.includes(key)) {
                                  return current;
                                }
                                return [...current, key];
                              }

                              return current.filter((value) => value !== key);
                            });
                          }}
                        />
                        <span>{repository.repo}</span>
                        <span className="tag">{repository.viewerPermission ?? "unknown"}</span>
                      </label>
                    );
                  })}

                  {visible.length === 0 ? (
                    <p className="info-text">{t("repo_selection.repo.empty")}</p>
                  ) : null}
                </div>
              </article>
            );
          })}
        </div>

        <div className="row gap-sm wrap">
          <button type="button" className="btn" onClick={confirmSelection}>
            {fmt("repo_selection.confirm", { count: selectedRepositoryCount })}
          </button>
        </div>
      </section>
    </section>
  );
}

function parseRepositories(owner: string, rows: unknown[]): ScopedRepositoryTarget[] {
  if (!Array.isArray(rows)) {
    return [];
  }

  const repositories: ScopedRepositoryTarget[] = [];
  for (const row of rows) {
    const record = asRecord(row);
    const nameWithOwner = asString(record.nameWithOwner);
    const repoFromNameWithOwner =
      nameWithOwner && nameWithOwner.includes("/") ? nameWithOwner.split("/")[1] : null;
    const repoName = repoFromNameWithOwner ?? asString(record.name);

    if (!repoName) {
      continue;
    }

    repositories.push({
      owner,
      repo: repoName,
      viewerPermission: normalizeViewerPermission(asString(record.viewerPermission) ?? undefined),
    });
  }

  return dedupeRepositories(repositories).sort((left, right) => left.repo.localeCompare(right.repo));
}

function dedupeStrings(values: string[]): string[] {
  const next: string[] = [];
  for (const value of values) {
    const trimmed = value.trim();
    if (trimmed.length === 0) {
      continue;
    }

    if (!next.includes(trimmed)) {
      next.push(trimmed);
    }
  }

  return next;
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

function formatTemplate(template: string, vars: Record<string, string | number>): string {
  return Object.entries(vars).reduce((output, [key, value]) => {
    return output.replaceAll(`{${key}}`, String(value));
  }, template);
}
