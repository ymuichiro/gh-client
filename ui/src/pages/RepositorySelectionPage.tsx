import { useCallback, useMemo, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Check, RefreshCw } from "lucide-react";

import { executeCommand } from "../core/executor";
import { useI18n } from "../core/i18n";
import { normalizeViewerPermission } from "../core/permissions";
import { IconButton } from "../components/IconButton";
import { LoadingIndicator } from "../components/LoadingIndicator";
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

interface RepoFetchProgress {
  processed: number;
  total: number;
}

interface OwnerRepoCacheEntry {
  updatedAt: number;
  repositories: ScopedRepositoryTarget[];
}

const OWNER_REPO_CACHE_TTL_MS = 5 * 60 * 1000;
const MAX_CONCURRENT_OWNER_FETCH = 4;

export function RepositorySelectionPage({
  initialConfig,
  onApplyConfig,
}: RepositorySelectionPageProps): JSX.Element {
  const { t } = useI18n();
  const navigate = useNavigate();
  const repoCacheByOwnerRef = useRef<Record<string, OwnerRepoCacheEntry>>(
    buildInitialOwnerCache(initialConfig),
  );

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
  const [repoFetchProgress, setRepoFetchProgress] = useState<RepoFetchProgress | null>(null);
  const [repoError, setRepoError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");

  const fmt = useCallback(
    (key: string, vars: Record<string, string | number>) => formatTemplate(t(key), vars),
    [t],
  );

  const loadOrganizations = useCallback(async () => {
    setOrgLoading(true);
    setOrgError(null);
    await waitForNextFrame();

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

    setRepoError(null);
    const now = Date.now();
    const cachedByOwner: Record<string, ScopedRepositoryTarget[]> = {};
    const ownersToFetch: string[] = [];

    for (const owner of selectedOrgs) {
      const cached = repoCacheByOwnerRef.current[owner];
      if (cached && now - cached.updatedAt < OWNER_REPO_CACHE_TTL_MS) {
        cachedByOwner[owner] = cached.repositories;
      } else {
        ownersToFetch.push(owner);
      }
    }

    if (ownersToFetch.length === 0) {
      const nextByOwner = Object.fromEntries(
        selectedOrgs.map((owner) => [owner, cachedByOwner[owner] ?? []]),
      );
      const availableKeys = new Set(
        Object.values(nextByOwner)
          .flat()
          .map((repository) => toRepositoryKey(repository.owner, repository.repo)),
      );

      setRepositoriesByOwner(nextByOwner);
      setSelectedRepoKeys((current) => current.filter((key) => availableKeys.has(key)));
      return;
    }

    setRepoLoading(true);
    setRepoFetchProgress({ processed: 0, total: ownersToFetch.length });
    await waitForNextFrame();

    try {
      const listed = await mapWithConcurrency(
        ownersToFetch,
        MAX_CONCURRENT_OWNER_FETCH,
        async (owner) => {
          const response = await executeCommand<unknown[]>(
            "repo.list",
            { owner, limit: 100 },
            { permission: "viewer" },
          );

          return [owner, parseRepositories(owner, response.data)] as const;
        },
        (processed, total) => {
          setRepoFetchProgress({ processed, total });
        },
      );

      const fetchedByOwner = Object.fromEntries(listed);
      const refreshedAt = Date.now();
      for (const [owner, repositories] of listed) {
        repoCacheByOwnerRef.current[owner] = {
          updatedAt: refreshedAt,
          repositories,
        };
      }

      const nextByOwner = Object.fromEntries(
        selectedOrgs.map((owner) => [
          owner,
          fetchedByOwner[owner] ??
            cachedByOwner[owner] ??
            repoCacheByOwnerRef.current[owner]?.repositories ??
            [],
        ]),
      );
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
      setRepoFetchProgress(null);
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
  const pageLoadingLabel = repoLoading
    ? repoFetchProgress
      ? fmt("repo_selection.repo.progress", {
          processed: repoFetchProgress.processed,
          total: repoFetchProgress.total,
        })
      : t("repo_selection.repo.loading")
    : orgLoading
      ? t("repo_selection.org.loading")
      : null;

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
    <section className="page-section loading-host">
      {pageLoadingLabel ? <LoadingIndicator overlay label={pageLoadingLabel} /> : null}
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
          <IconButton
            icon={RefreshCw}
            label={orgLoading ? t("repo_selection.org.loading") : t("repo_selection.org.reload")}
            variant="secondary"
            onClick={() => void loadOrganizations()}
            disabled={orgLoading}
          />
          <IconButton
            icon={RefreshCw}
            label={repoLoading ? t("repo_selection.repo.loading") : t("repo_selection.repo.update")}
            variant="primary"
            onClick={() => {
              void updateRepositoryCandidates();
            }}
            disabled={repoLoading || selectedOrgs.length === 0}
          />
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
            disabled={repoLoading}
          />
        </label>

        {repoError ? <p className="error-text">{repoError}</p> : null}
        {repoLoading && repoFetchProgress ? (
          <p className="info-text">
            {fmt("repo_selection.repo.progress", {
              processed: repoFetchProgress.processed,
              total: repoFetchProgress.total,
            })}
          </p>
        ) : null}

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
          <IconButton
            icon={Check}
            label={fmt("repo_selection.confirm", { count: selectedRepositoryCount })}
            variant="primary"
            onClick={confirmSelection}
            disabled={orgLoading || repoLoading}
          />
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
  const seen = new Set<string>();
  const next: string[] = [];
  for (const value of values) {
    const trimmed = value.trim();
    if (trimmed.length === 0) {
      continue;
    }

    if (seen.has(trimmed)) {
      continue;
    }

    seen.add(trimmed);
    next.push(trimmed);
  }

  return next;
}

function buildInitialOwnerCache(
  config: RepositoryScopeConfig | null,
): Record<string, OwnerRepoCacheEntry> {
  if (!config) {
    return {};
  }

  const grouped = groupRepositoriesByOwner(config.repositories);
  const updatedAt = Date.parse(config.updatedAt);
  const cacheUpdatedAt = Number.isFinite(updatedAt) ? updatedAt : 0;
  const cache: Record<string, OwnerRepoCacheEntry> = {};
  for (const [owner, ownerRepos] of Object.entries(grouped)) {
    cache[owner] = {
      updatedAt: cacheUpdatedAt,
      repositories: ownerRepos,
    };
  }
  return cache;
}

async function mapWithConcurrency<T, R>(
  values: T[],
  concurrency: number,
  mapper: (value: T) => Promise<R>,
  onProgress?: (processed: number, total: number) => void,
): Promise<R[]> {
  if (values.length === 0) {
    return [];
  }

  const workers = Math.max(1, Math.min(concurrency, values.length));
  const results: R[] = new Array(values.length);
  let cursor = 0;
  let processed = 0;

  const run = async () => {
    while (true) {
      const index = cursor;
      cursor += 1;
      if (index >= values.length) {
        return;
      }

      results[index] = await mapper(values[index]);
      processed += 1;
      onProgress?.(processed, values.length);
    }
  };

  await Promise.all(Array.from({ length: workers }, () => run()));
  return results;
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

function waitForNextFrame(): Promise<void> {
  return new Promise((resolve) => {
    if (typeof window === "undefined" || typeof window.requestAnimationFrame !== "function") {
      setTimeout(resolve, 0);
      return;
    }

    window.requestAnimationFrame(() => resolve());
  });
}
