import { Suspense, lazy, useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  AlignJustify,
  ArrowLeft,
  Check,
  CircleX,
  Code2,
  Eraser,
  ExternalLink,
  GitMerge,
  MessageSquare,
  RefreshCw,
  RotateCcw,
  Save,
  SplitSquareHorizontal,
  Tag,
  Trash2,
  Undo2,
  UserMinus,
  UserPlus,
} from "lucide-react";

import { type CommandExecutionEvent } from "../components/CommandForm";
import { IconButton } from "../components/IconButton";
import { InboxActionModal, type InboxActionModalField } from "../components/InboxActionModal";
import { LoadingIndicator } from "../components/LoadingIndicator";
import type { DiffViewMode } from "../components/DiffSyntaxPreview";
import type { CommandId } from "../core/commandIds";
import { ExecutionError, executeCommand } from "../core/executor";
import { openExternalUrl } from "../core/externalOpen";
import { loadHistory } from "../core/history";
import { useI18n } from "../core/i18n";
import type { CommandPermission, FrontendInvokeError } from "../core/types";
import {
  INBOX_CACHE_TTL_MS,
  isInboxCacheStale,
  readInboxCache,
  writeInboxCache,
} from "./inboxCache";
import {
  type ActionGuardReason,
  type BatchExecutionOutcome,
  type BatchExecutionResult,
  type InboxActionType,
  type InboxFilters,
  type InboxItem,
  type ItemKind,
  type SortMode,
  appendBatchExecutionResult,
  asNumber,
  asRecord,
  asString,
  createBatchExecutionResult,
  deriveItemTiming,
  evaluateActionGuard,
  extractNamedValues,
  extractReviewerLogins,
  filterAndSortItems,
  formatUpdatedLabel,
  initialInboxFilters,
  normalizeMergeMethod,
  parseCommaSeparated,
  parsePositiveNumber,
  toRepoKey,
} from "./inboxLogic";

const MAX_CONCURRENT_REPO_FETCH = 4;
const BATCH_PREVIEW_LIMIT = 40;
const LazyDiffSyntaxPreview = lazy(() =>
  import("../components/DiffSyntaxPreview").then((module) => ({
    default: module.DiffSyntaxPreview,
  })),
);

interface RepoTarget {
  owner: string;
  repo: string;
  viewerPermission: CommandPermission | null;
}

interface SavedView {
  id: string;
  name: string;
  repoKeys: string[];
  filters: InboxFilters;
  sortMode: SortMode;
  builtIn: boolean;
}

interface PullRequestComment {
  id?: number | null;
  kind?: string;
  body: string;
  created_at?: string;
  author?: { login?: string } | null;
}

interface PullRequestReviewThread {
  thread_id: string;
  is_resolved: boolean;
  is_outdated: boolean;
  path?: string | null;
  line?: number | null;
  comments?: PullRequestComment[];
}

interface PullRequestDiffFile {
  filename: string;
  status: string;
  additions: number;
  deletions: number;
  patch?: string | null;
}

interface PullRequestDetail {
  number: number;
  title: string;
  body: string;
  state: string;
  url: string;
  isDraft?: boolean;
  mergeStateStatus?: string | null;
  reviewDecision?: string | null;
  additions?: number;
  deletions?: number;
  changedFiles?: number;
}

interface PullRequestRawDiff {
  text: string;
}

interface PullRequestDetailState {
  loading: boolean;
  error: string | null;
  warning: string | null;
  detail: PullRequestDetail | null;
  comments: PullRequestComment[];
  threads: PullRequestReviewThread[];
  diffFiles: PullRequestDiffFile[];
  rawDiffText: string;
}

interface IssueCommentEntry {
  id?: number | null;
  body: string;
  createdAt?: string | null;
  author?: { login?: string } | null;
  url?: string | null;
}

interface IssueDetail {
  number: number;
  title: string;
  state: string;
  url: string;
  body: string;
  comments: IssueCommentEntry[];
  assignees?: Array<{ login?: string }>;
  labels?: Array<{ name?: string }>;
  updatedAt?: string | null;
}

interface IssueDetailState {
  loading: boolean;
  error: string | null;
  detail: IssueDetail | null;
}

interface RepoFetchResult {
  key: string;
  items: InboxItem[];
  errors: string[];
}

interface BatchProgress {
  total: number;
  processed: number;
}

interface InboxActionRequest {
  commandId: CommandId;
  payload: Record<string, unknown>;
  permission: CommandPermission;
}

type IssueEditField =
  | "add_assignees"
  | "remove_assignees"
  | "add_labels"
  | "remove_labels";

type ModalState =
  | { kind: "none" }
  | { kind: "save_view" }
  | { kind: "approve"; item: InboxItem }
  | { kind: "comment"; item: InboxItem; initialBody?: string }
  | { kind: "request_changes"; item: InboxItem }
  | { kind: "merge"; item: InboxItem }
  | { kind: "issue_edit"; item: InboxItem; field: IssueEditField }
  | { kind: "batch_close"; targets: InboxItem[] };

interface InboxPageProps {
  repoTargets: RepoTarget[];
  onExecuted: (event: CommandExecutionEvent) => void;
  onInspect: (title: string, value: unknown) => void;
  mode: ItemKind;
  title: string;
  subtitle: string;
}

const initialPrDetailState: PullRequestDetailState = {
  loading: false,
  error: null,
  warning: null,
  detail: null,
  comments: [],
  threads: [],
  diffFiles: [],
  rawDiffText: "",
};

const initialIssueDetailState: IssueDetailState = {
  loading: false,
  error: null,
  detail: null,
};

export function InboxPage({
  repoTargets,
  onExecuted,
  onInspect,
  mode,
  title,
  subtitle,
}: InboxPageProps): JSX.Element {
  const { t } = useI18n();
  const savedViewsStorageKey = useMemo(() => getSavedViewsStorageKey(mode), [mode]);
  const [selectedRepoKeys, setSelectedRepoKeys] = useState<string[]>([]);
  const [filters, setFilters] = useState<InboxFilters>(() => initialInboxFilters());
  const [sortMode, setSortMode] = useState<SortMode>("priority");
  const [slaHours, setSlaHours] = useState("24");
  const [customViews, setCustomViews] = useState<SavedView[]>(() =>
    loadCustomViews(getSavedViewsStorageKey(mode)),
  );
  const [selectedViewId, setSelectedViewId] = useState<string>("");
  const [items, setItems] = useState<InboxItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [fetchWarnings, setFetchWarnings] = useState<string[]>([]);
  const [lastLoadedAt, setLastLoadedAt] = useState<number | null>(null);
  const [selectedItemId, setSelectedItemId] = useState("");
  const [checkedItemIds, setCheckedItemIds] = useState<string[]>([]);
  const [detailOpen, setDetailOpen] = useState(false);
  const [selectedDiffPath, setSelectedDiffPath] = useState<string | null>(null);
  const [diffViewMode, setDiffViewMode] = useState<DiffViewMode>("inline");
  const [mutationRunning, setMutationRunning] = useState(false);
  const [batchRunning, setBatchRunning] = useState(false);
  const [batchProgress, setBatchProgress] = useState<BatchProgress | null>(null);
  const [batchResult, setBatchResult] = useState<BatchExecutionResult | null>(null);
  const [prDetailState, setPrDetailState] = useState<PullRequestDetailState>(initialPrDetailState);
  const [issueDetailState, setIssueDetailState] = useState<IssueDetailState>(initialIssueDetailState);
  const [modalState, setModalState] = useState<ModalState>({ kind: "none" });
  const [modalRunning, setModalRunning] = useState(false);
  const [modalError, setModalError] = useState<string | null>(null);
  const [currentLogin, setCurrentLogin] = useState<string | null>(null);

  const inboxFetchSeq = useRef(0);
  const prDetailFetchSeq = useRef(0);
  const issueDetailFetchSeq = useRef(0);

  const defaultViews = useMemo(() => buildDefaultViews(t, mode), [t, mode]);
  const allViews = useMemo(() => [...defaultViews, ...customViews], [customViews, defaultViews]);

  const repoTargetsByKey = useMemo(() => {
    const next = new Map<string, RepoTarget>();
    for (const target of repoTargets) {
      next.set(toRepoKey(target.owner, target.repo), target);
    }
    return next;
  }, [repoTargets]);

  const repoEntries = useMemo(
    () => repoTargets.map((target) => ({ ...target, key: toRepoKey(target.owner, target.repo) })),
    [repoTargets],
  );

  const fmt = useCallback(
    (key: string, vars: Record<string, string | number>) => formatTemplate(t(key), vars),
    [t],
  );

  useEffect(() => {
    let active = true;

    void executeCommand<unknown>("auth.status", {}, { permission: "viewer" })
      .then((response) => {
        if (!active) {
          return;
        }

        const account = asString(asRecord(response.data).account);
        setCurrentLogin(account ?? null);
      })
      .catch(() => {
        if (!active) {
          return;
        }
        setCurrentLogin(null);
      });

    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    setSelectedRepoKeys((previous) => {
      const available = new Set(repoEntries.map((entry) => entry.key));
      const retained = previous.filter((key) => available.has(key));
      if (retained.length > 0) {
        return retained;
      }
      return repoEntries.map((entry) => entry.key);
    });
  }, [repoEntries]);

  useEffect(() => {
    setFilters(initialInboxFilters());
    setSortMode("priority");
    setSelectedViewId("");
    setCustomViews(loadCustomViews(savedViewsStorageKey));
  }, [savedViewsStorageKey]);

  useEffect(() => {
    const serializable = customViews.map((view) => ({
      id: view.id,
      name: view.name,
      repoKeys: view.repoKeys,
      filters: view.filters,
      sortMode: view.sortMode,
    }));

    try {
      localStorage.setItem(savedViewsStorageKey, JSON.stringify(serializable));
    } catch (_cause) {
      // ignore persistence failures in restricted runtime contexts
    }
  }, [customViews, savedViewsStorageKey]);

  const executeMutation = useCallback(
    async <T,>(
      commandId: CommandId,
      payload: Record<string, unknown>,
      permission: CommandPermission,
    ): Promise<T> => {
      try {
        const result = await executeCommand<T>(commandId, payload, { permission });
        onExecuted({
          commandId,
          requestId: result.requestId,
          payload: result.payload,
          status: "success",
          data: result.data,
        });
        return result.data;
      } catch (cause) {
        const detail = toFrontendInvokeError(cause, commandId);
        onExecuted({
          commandId,
          requestId: detail.request_id,
          payload,
          status: "error",
          error: detail,
        });
        throw cause;
      }
    },
    [onExecuted],
  );

  const loadRepoInbox = useCallback(
    async (target: RepoTarget, forceRefresh: boolean): Promise<RepoFetchResult> => {
      const permission = target.viewerPermission ?? "viewer";
      const repoKey = toRepoKey(target.owner, target.repo);

      if (mode === "pr") {
        const prSettled = await Promise.allSettled([
          executeCommand<unknown[]>(
            "pr.list",
            {
              owner: target.owner,
              repo: target.repo,
              limit: 100,
              force_refresh: forceRefresh,
            },
            { permission },
          ),
        ]);
        const nextItems: InboxItem[] = [];
        const errors: string[] = [];

        if (prSettled[0].status === "fulfilled") {
          nextItems.push(
            ...mapPullRequests(target.owner, target.repo, prSettled[0].value.data ?? []),
          );
        } else {
          errors.push(formatErrorForRepo("pr.list", prSettled[0].reason));
        }

        return {
          key: repoKey,
          items: nextItems,
          errors,
        };
      }

      const issueSettled = await Promise.allSettled([
        executeCommand<unknown[]>(
          "issue.list",
          {
            owner: target.owner,
            repo: target.repo,
            limit: 100,
            force_refresh: forceRefresh,
          },
          { permission },
        ),
      ]);

      const nextItems: InboxItem[] = [];
      const errors: string[] = [];

      if (issueSettled[0].status === "fulfilled") {
        nextItems.push(...mapIssues(target.owner, target.repo, issueSettled[0].value.data ?? []));
      } else {
        errors.push(formatErrorForRepo("issue.list", issueSettled[0].reason));
      }

      return {
        key: repoKey,
        items: nextItems,
        errors,
      };
    },
    [mode],
  );

  const resolvedSlaHours = useMemo(() => parsePositiveNumber(slaHours) ?? 24, [slaHours]);

  const modeItems = useMemo(
    () => items.filter((item) => item.kind === mode),
    [items, mode],
  );

  const effectiveFilters = useMemo<InboxFilters>(() => {
    if (mode === "pr") {
      return filters;
    }

    return {
      ...filters,
      reviewer: "",
      draft: "all",
      state: filters.state === "merged" ? "all" : filters.state,
    };
  }, [filters, mode]);

  const filteredItems = useMemo(
    () => filterAndSortItems(modeItems, effectiveFilters, sortMode, resolvedSlaHours, currentLogin),
    [currentLogin, effectiveFilters, modeItems, resolvedSlaHours, sortMode],
  );
  const checkedItemIdSet = useMemo(() => new Set(checkedItemIds), [checkedItemIds]);
  const selectedBatchTargets = useMemo(
    () => filteredItems.filter((item) => checkedItemIdSet.has(item.id)),
    [checkedItemIdSet, filteredItems],
  );

  const applyInboxSnapshot = useCallback(
    (nextItems: InboxItem[], warnings: string[], updatedAt: number | null) => {
      setItems(nextItems);
      setFetchWarnings(warnings);
      setLastLoadedAt(updatedAt);

      if (nextItems.length === 0 && warnings.length > 0) {
        setError(t("inbox.error.fetch_all_failed"));
        return;
      }

      setError(null);
    },
    [t],
  );

  const refreshInbox = useCallback(async (options?: { force?: boolean }) => {
    const forceRefresh = options?.force ?? false;
    const requestSeq = ++inboxFetchSeq.current;
    setLoading(true);
    setError(null);

    await waitForNextFrame();
    if (requestSeq !== inboxFetchSeq.current) {
      return;
    }

    try {
      const selectedTargets = selectedRepoKeys
        .map((key) => repoTargetsByKey.get(key))
        .filter((target): target is RepoTarget => Boolean(target));

      if (selectedTargets.length === 0) {
        applyInboxSnapshot([], [], Date.now());
        return;
      }

      const fetchResults = await mapWithConcurrency(
        selectedTargets,
        MAX_CONCURRENT_REPO_FETCH,
        (target) => loadRepoInbox(target, forceRefresh),
      );

      if (requestSeq !== inboxFetchSeq.current) {
        return;
      }

      const nextItems = fetchResults.flatMap((result) => result.items);
      const warnings = fetchResults.flatMap((result) =>
        result.errors.map((message) => `${result.key}: ${message}`),
      );

      const updatedAt = Date.now();
      applyInboxSnapshot(nextItems, warnings, updatedAt);
      writeInboxCache(mode, selectedRepoKeys, {
        items: nextItems,
        warnings,
        updatedAt,
      });
    } catch (cause) {
      if (requestSeq !== inboxFetchSeq.current) {
        return;
      }
      setError(toErrorMessage(cause));
    } finally {
      if (requestSeq === inboxFetchSeq.current) {
        setLoading(false);
      }
    }
  }, [applyInboxSnapshot, loadRepoInbox, mode, repoTargetsByKey, selectedRepoKeys]);

  useEffect(() => {
    if (selectedRepoKeys.length === 0) {
      applyInboxSnapshot([], [], null);
      return;
    }

    const cached = readInboxCache(mode, selectedRepoKeys);
    if (cached) {
      applyInboxSnapshot(cached.items, cached.warnings, cached.updatedAt);
      if (isInboxCacheStale(cached)) {
        void refreshInbox({ force: true });
      }
      return;
    }

    void refreshInbox({ force: true });
  }, [applyInboxSnapshot, mode, refreshInbox, selectedRepoKeys]);

  useEffect(() => {
    if (selectedRepoKeys.length === 0) {
      return;
    }

    const timer = window.setInterval(() => {
      if (loading || !lastLoadedAt) {
        return;
      }

      if (Date.now() - lastLoadedAt < INBOX_CACHE_TTL_MS) {
        return;
      }

      void refreshInbox({ force: true });
    }, 30_000);

    return () => window.clearInterval(timer);
  }, [lastLoadedAt, loading, refreshInbox, selectedRepoKeys.length]);

  const selectedItem = useMemo(() => {
    if (!selectedItemId) {
      return null;
    }

    return modeItems.find((item) => item.id === selectedItemId) ?? null;
  }, [modeItems, selectedItemId]);

  const resolvePermissionForItem = useCallback(
    (item: InboxItem | null): CommandPermission | null => {
      if (!item) {
        return null;
      }

      return (
        repoTargetsByKey.get(toRepoKey(item.owner, item.repo))?.viewerPermission ?? "viewer"
      );
    },
    [repoTargetsByKey],
  );

  useEffect(() => {
    if (!selectedItemId) {
      return;
    }

    if (selectedItem) {
      return;
    }

    setSelectedItemId("");
    setDetailOpen(false);
  }, [selectedItem, selectedItemId]);

  useEffect(() => {
    const modeItemIdSet = new Set(modeItems.map((item) => item.id));
    setCheckedItemIds((current) => current.filter((id) => modeItemIdSet.has(id)));
  }, [modeItems]);

  const selectedPermission = useMemo(() => {
    return resolvePermissionForItem(selectedItem);
  }, [resolvePermissionForItem, selectedItem]);

  useEffect(() => {
    const target = selectedItem;
    if (!detailOpen || !target || target.kind !== "pr") {
      prDetailFetchSeq.current += 1;
      setPrDetailState(initialPrDetailState);
      setSelectedDiffPath(null);
      return;
    }

    const permission =
      repoTargetsByKey.get(toRepoKey(target.owner, target.repo))?.viewerPermission ?? "viewer";
    const requestSeq = ++prDetailFetchSeq.current;

    setPrDetailState({ ...initialPrDetailState, loading: true });

    void Promise.allSettled([
      executeCommand<PullRequestDetail>(
        "pr.view",
        { owner: target.owner, repo: target.repo, number: target.number },
        { permission },
      ),
      executeCommand<PullRequestComment[]>(
        "pr.comments.list",
        { owner: target.owner, repo: target.repo, number: target.number },
        { permission },
      ),
      executeCommand<PullRequestReviewThread[]>(
        "pr.review_threads.list",
        { owner: target.owner, repo: target.repo, number: target.number },
        { permission },
      ),
      executeCommand<PullRequestDiffFile[]>(
        "pr.diff.files.list",
        { owner: target.owner, repo: target.repo, number: target.number },
        { permission },
      ),
      executeCommand<PullRequestRawDiff>(
        "pr.diff.raw.get",
        { owner: target.owner, repo: target.repo, number: target.number },
        { permission },
      ),
    ]).then((settled) => {
      if (requestSeq !== prDetailFetchSeq.current) {
        return;
      }

      const [detailSettled, commentsSettled, threadsSettled, filesSettled, rawDiffSettled] = settled;

      const detail =
        detailSettled.status === "fulfilled" ? detailSettled.value.data : null;
      const comments =
        commentsSettled.status === "fulfilled" ? commentsSettled.value.data ?? [] : [];
      const threads =
        threadsSettled.status === "fulfilled" ? threadsSettled.value.data ?? [] : [];
      const diffFiles = filesSettled.status === "fulfilled" ? filesSettled.value.data ?? [] : [];
      const rawDiffText =
        rawDiffSettled.status === "fulfilled" ? rawDiffSettled.value.data?.text ?? "" : "";

      const failureCount = settled.filter((entry) => entry.status === "rejected").length;

      setPrDetailState({
        loading: false,
        error: detail ? null : t("inbox.pr.error.detail_unavailable"),
        warning:
          detail && failureCount > 0
            ? fmt("inbox.pr.warning.partial", { count: failureCount })
            : null,
        detail,
        comments,
        threads,
        diffFiles,
        rawDiffText,
      });

      setSelectedDiffPath((current) => {
        if (current && diffFiles.some((file) => file.filename === current)) {
          return current;
        }

        return diffFiles[0]?.filename ?? null;
      });
    });
  }, [detailOpen, fmt, repoTargetsByKey, selectedItem, t]);

  useEffect(() => {
    const target = selectedItem;
    if (!detailOpen || !target || target.kind !== "issue") {
      issueDetailFetchSeq.current += 1;
      setIssueDetailState(initialIssueDetailState);
      return;
    }

    const permission =
      repoTargetsByKey.get(toRepoKey(target.owner, target.repo))?.viewerPermission ?? "viewer";
    const requestSeq = ++issueDetailFetchSeq.current;

    setIssueDetailState({ loading: true, error: null, detail: null });

    void executeCommand<IssueDetail>(
      "issue.view",
      { owner: target.owner, repo: target.repo, number: target.number },
      { permission },
    )
      .then((result) => {
        if (requestSeq !== issueDetailFetchSeq.current) {
          return;
        }

        setIssueDetailState({
          loading: false,
          error: null,
          detail: normalizeIssueDetail(result.data),
        });
      })
      .catch((cause) => {
        if (requestSeq !== issueDetailFetchSeq.current) {
          return;
        }

        setIssueDetailState({
          loading: false,
          error: toErrorMessage(cause),
          detail: null,
        });
      });
  }, [detailOpen, repoTargetsByKey, selectedItem]);

  const selectedDiffFile = useMemo(
    () => prDetailState.diffFiles.find((file) => file.filename === selectedDiffPath) ?? null,
    [prDetailState.diffFiles, selectedDiffPath],
  );

  const unresolvedThreads = useMemo(
    () => prDetailState.threads.filter((thread) => !thread.is_resolved),
    [prDetailState.threads],
  );

  const recentActivity = useMemo(() => loadHistory().slice(0, 8), []);

  const guardedReasonForSelected = useCallback(
    (action: InboxActionType): ActionGuardReason | null => {
      if (!selectedItem) {
        return "permission_required";
      }

      return evaluateActionGuard(selectedItem, selectedPermission, action);
    },
    [selectedItem, selectedPermission],
  );

  const runSingleAction = useCallback(
    async (request: InboxActionRequest, refreshAfter = true): Promise<boolean> => {
      setMutationRunning(true);
      setError(null);

      try {
        await executeMutation(request.commandId, request.payload, request.permission);
        if (refreshAfter) {
          await refreshInbox({ force: true });
        }
        return true;
      } catch (cause) {
        setError(toErrorMessage(cause));
        return false;
      } finally {
        setMutationRunning(false);
      }
    },
    [executeMutation, refreshInbox],
  );

  const closeSelected = useCallback(async () => {
    if (!selectedItem || !selectedPermission) {
      return;
    }

    const guard = evaluateActionGuard(selectedItem, selectedPermission, "close");
    if (guard) {
      setError(actionGuardMessage(t, guard));
      return;
    }

    if (selectedItem.kind === "issue") {
      await runSingleAction({
        commandId: "issue.close",
        permission: selectedPermission,
        payload: {
          owner: selectedItem.owner,
          repo: selectedItem.repo,
          number: selectedItem.number,
          reason: "completed",
        },
      });
      return;
    }

    await runSingleAction({
      commandId: "pr.close",
      permission: selectedPermission,
      payload: {
        owner: selectedItem.owner,
        repo: selectedItem.repo,
        number: selectedItem.number,
      },
    });
  }, [runSingleAction, selectedItem, selectedPermission, t]);

  const reopenSelected = useCallback(async () => {
    if (!selectedItem || !selectedPermission) {
      return;
    }

    const guard = evaluateActionGuard(selectedItem, selectedPermission, "reopen");
    if (guard) {
      setError(actionGuardMessage(t, guard));
      return;
    }

    if (selectedItem.kind === "issue") {
      await runSingleAction({
        commandId: "issue.reopen",
        permission: selectedPermission,
        payload: {
          owner: selectedItem.owner,
          repo: selectedItem.repo,
          number: selectedItem.number,
        },
      });
      return;
    }

    await runSingleAction({
      commandId: "pr.reopen",
      permission: selectedPermission,
      payload: {
        owner: selectedItem.owner,
        repo: selectedItem.repo,
        number: selectedItem.number,
      },
    });
  }, [runSingleAction, selectedItem, selectedPermission, t]);

  const openModal = useCallback((nextState: ModalState) => {
    setModalError(null);
    setModalState(nextState);
  }, []);

  const closeModal = useCallback(() => {
    if (modalRunning) {
      return;
    }
    setModalState({ kind: "none" });
    setModalError(null);
  }, [modalRunning]);

  const runBatchClose = useCallback(
    async (
      targets: InboxItem[],
      issueCloseReason: "completed" | "not planned",
    ): Promise<BatchExecutionResult> => {
      let result = createBatchExecutionResult(targets.length);

      setBatchRunning(true);
      setBatchProgress({ total: targets.length, processed: 0 });

      for (const target of targets) {
        const permission = resolvePermissionForItem(target);
        const guard = evaluateActionGuard(target, permission, "close");
        let outcome: BatchExecutionOutcome = "success";

        if (guard || !permission) {
          outcome = "skipped";
          result = appendBatchExecutionResult(result, outcome);
          setBatchProgress({ total: targets.length, processed: result.processed });
          continue;
        }

        try {
          if (target.kind === "issue") {
            await executeMutation(
              "issue.close",
              {
                owner: target.owner,
                repo: target.repo,
                number: target.number,
                reason: issueCloseReason,
              },
              permission,
            );
          } else {
            await executeMutation(
              "pr.close",
              {
                owner: target.owner,
                repo: target.repo,
                number: target.number,
              },
              permission,
            );
          }
        } catch (_cause) {
          outcome = "failed";
        }

        result = appendBatchExecutionResult(result, outcome);
        setBatchProgress({ total: targets.length, processed: result.processed });
      }

      setBatchRunning(false);
      setBatchProgress(null);
      setBatchResult(result);
      setCheckedItemIds((current) => {
        const targetIdSet = new Set(targets.map((target) => target.id));
        return current.filter((id) => !targetIdSet.has(id));
      });
      await refreshInbox({ force: true });
      return result;
    },
    [executeMutation, refreshInbox, resolvePermissionForItem],
  );

  const handleModalConfirm = useCallback(
    async (values: Record<string, string>) => {
      const modal = modalState;
      setModalRunning(true);
      setModalError(null);
      setError(null);

      try {
        switch (modal.kind) {
          case "save_view": {
            const name = values.name?.trim() ?? "";
            if (!name) {
              setModalError(t("inbox.modal.validation.required"));
              return;
            }

            const id =
              typeof crypto !== "undefined" && crypto.randomUUID
                ? crypto.randomUUID()
                : `${Date.now()}`;

            const nextView: SavedView = {
              id,
              name,
              repoKeys: selectedRepoKeys,
              filters,
              sortMode,
              builtIn: false,
            };

            setCustomViews((current) => [nextView, ...current]);
            setSelectedViewId(id);
            setModalState({ kind: "none" });
            return;
          }
          case "approve": {
            const permission = resolvePermissionForItem(modal.item);
            const guard = evaluateActionGuard(modal.item, permission, "approve");
            if (guard || !permission) {
              setModalError(actionGuardMessage(t, guard ?? "permission_required"));
              return;
            }

            const ok = await runSingleAction({
              commandId: "pr.review",
              permission,
              payload: {
                owner: modal.item.owner,
                repo: modal.item.repo,
                number: modal.item.number,
                event: "approve",
              },
            });
            if (ok) {
              setModalState({ kind: "none" });
            }
            return;
          }
          case "comment": {
            const permission = resolvePermissionForItem(modal.item);
            const guard = evaluateActionGuard(modal.item, permission, "comment");
            if (guard || !permission) {
              setModalError(actionGuardMessage(t, guard ?? "permission_required"));
              return;
            }

            const body = values.body?.trim() ?? "";
            if (!body) {
              setModalError(t("inbox.modal.validation.required"));
              return;
            }

            const ok =
              modal.item.kind === "issue"
                ? await runSingleAction({
                    commandId: "issue.comment",
                    permission,
                    payload: {
                      owner: modal.item.owner,
                      repo: modal.item.repo,
                      number: modal.item.number,
                      body,
                    },
                  })
                : await runSingleAction({
                    commandId: "pr.review",
                    permission,
                    payload: {
                      owner: modal.item.owner,
                      repo: modal.item.repo,
                      number: modal.item.number,
                      event: "comment",
                      body,
                    },
                  });

            if (ok) {
              setModalState({ kind: "none" });
            }
            return;
          }
          case "request_changes": {
            const permission = resolvePermissionForItem(modal.item);
            const guard = evaluateActionGuard(
              modal.item,
              permission,
              "request_changes",
            );
            if (guard || !permission) {
              setModalError(actionGuardMessage(t, guard ?? "permission_required"));
              return;
            }

            const body = values.body?.trim() ?? "";
            if (!body) {
              setModalError(t("inbox.modal.validation.required"));
              return;
            }

            const ok = await runSingleAction({
              commandId: "pr.review",
              permission,
              payload: {
                owner: modal.item.owner,
                repo: modal.item.repo,
                number: modal.item.number,
                event: "request_changes",
                body,
              },
            });

            if (ok) {
              setModalState({ kind: "none" });
            }
            return;
          }
          case "merge": {
            const permission = resolvePermissionForItem(modal.item);
            const guard = evaluateActionGuard(modal.item, permission, "merge");
            if (guard || !permission) {
              setModalError(actionGuardMessage(t, guard ?? "permission_required"));
              return;
            }

            const method = normalizeMergeMethod(values.method ?? "squash");
            const ok = await runSingleAction({
              commandId: "pr.merge",
              permission,
              payload: {
                owner: modal.item.owner,
                repo: modal.item.repo,
                number: modal.item.number,
                method,
                delete_branch: false,
              },
            });

            if (ok) {
              setModalState({ kind: "none" });
            }
            return;
          }
          case "issue_edit": {
            const permission = resolvePermissionForItem(modal.item);
            const guard = evaluateActionGuard(modal.item, permission, "issue_edit");
            if (guard || !permission) {
              setModalError(actionGuardMessage(t, guard ?? "permission_required"));
              return;
            }

            const parsed = (() => {
              if (
                modal.field === "add_assignees" ||
                modal.field === "remove_assignees"
              ) {
                const selected = parseCommaSeparated(values.selected_values ?? "");
                const manual = parseCommaSeparated(values.manual_values ?? "");
                return dedupeStrings([...selected, ...manual]);
              }

              const raw = values.values?.trim() ?? "";
              return parseCommaSeparated(raw);
            })();

            if (parsed.length === 0) {
              setModalError(t("inbox.modal.validation.required"));
              return;
            }

            const ok = await runSingleAction({
              commandId: "issue.edit",
              permission,
              payload: {
                owner: modal.item.owner,
                repo: modal.item.repo,
                number: modal.item.number,
                [modal.field]: parsed,
              },
            });

            if (ok) {
              setModalState({ kind: "none" });
            }
            return;
          }
          case "batch_close": {
            const issueReasonRaw = values.issue_reason === "not planned" ? "not planned" : "completed";
            await runBatchClose(modal.targets, issueReasonRaw);
            setModalState({ kind: "none" });
            return;
          }
          default:
            return;
        }
      } catch (cause) {
        setModalError(toErrorMessage(cause));
      } finally {
        setModalRunning(false);
      }
    },
    [
      filters,
      modalState,
      runBatchClose,
      runSingleAction,
      resolvePermissionForItem,
      selectedRepoKeys,
      sortMode,
      t,
    ],
  );

  const openBatchCloseModal = useCallback(() => {
    const targets = selectedBatchTargets;
    if (targets.length === 0) {
      setError(t("inbox.batch.no_selection"));
      return;
    }

    openModal({ kind: "batch_close", targets });
  }, [openModal, selectedBatchTargets, t]);

  useEffect(() => {
    const hasModal = modalState.kind !== "none";

    const onKeyDown = (event: KeyboardEvent) => {
      if (hasModal || mutationRunning || modalRunning || !detailOpen) {
        return;
      }

      if (event.ctrlKey || event.metaKey || event.altKey) {
        return;
      }

      const target = event.target as HTMLElement | null;
      if (target && ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName)) {
        return;
      }

      if (target?.isContentEditable) {
        return;
      }

      if (!selectedItem) {
        return;
      }

      const key = event.key.toLowerCase();

      if (key === "a") {
        const guard = evaluateActionGuard(selectedItem, selectedPermission, "approve");
        if (!guard && selectedItem.kind === "pr") {
          event.preventDefault();
          openModal({ kind: "approve", item: selectedItem });
        }
        return;
      }

      if (key === "c") {
        const guard = evaluateActionGuard(selectedItem, selectedPermission, "close");
        if (!guard) {
          event.preventDefault();
          void closeSelected();
        }
        return;
      }

      if (key === "r") {
        const guard = evaluateActionGuard(selectedItem, selectedPermission, "comment");
        if (!guard) {
          event.preventDefault();
          openModal({ kind: "comment", item: selectedItem });
        }
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [
    closeSelected,
    detailOpen,
    modalRunning,
    modalState.kind,
    mutationRunning,
    openModal,
    selectedItem,
    selectedPermission,
  ]);

  const diffPreviewText =
    selectedDiffFile?.patch?.trim().length
      ? selectedDiffFile.patch
      : prDetailState.rawDiffText;

  const issueEditAssigneeOptions = useMemo(() => {
    if (modalState.kind !== "issue_edit") {
      return [] as string[];
    }

    if (
      modalState.field !== "add_assignees" &&
      modalState.field !== "remove_assignees"
    ) {
      return [] as string[];
    }

    const repoKey = toRepoKey(modalState.item.owner, modalState.item.repo);
    const known = new Set<string>();

    for (const item of items) {
      if (toRepoKey(item.owner, item.repo) !== repoKey) {
        continue;
      }

      if (item.author) {
        known.add(item.author);
      }
      for (const assignee of item.assignees) {
        known.add(assignee);
      }
    }

    for (const assignee of modalState.item.assignees) {
      known.add(assignee);
    }

    if (issueDetailState.detail && issueDetailState.detail.number === modalState.item.number) {
      for (const assignee of issueDetailState.detail.assignees ?? []) {
        const login = asString(assignee.login);
        if (login) {
          known.add(login);
        }
      }

      for (const comment of issueDetailState.detail.comments) {
        const login = asString(asRecord(comment.author).login);
        if (login) {
          known.add(login);
        }
      }
    }

    const currentAssignees = new Set(modalState.item.assignees.map((assignee) => assignee.toLowerCase()));
    const options = [...known].sort((left, right) => left.localeCompare(right));

    if (modalState.field === "remove_assignees") {
      return options.filter((login) => currentAssignees.has(login.toLowerCase()));
    }

    return options.filter((login) => !currentAssignees.has(login.toLowerCase()));
  }, [issueDetailState.detail, items, modalState]);

  const modalConfig = buildModalConfig(
    modalState,
    t,
    fmt,
    selectedRepoKeys,
    filters,
    sortMode,
    issueEditAssigneeOptions,
  );

  return (
    <section className="inbox-page loading-host">
      <aside className="inbox-panel inbox-left">
        <header className="section-header">
          <h2>{title}</h2>
          <p>{subtitle}</p>
        </header>

        <div className="row gap-sm">
          <IconButton
            icon={RefreshCw}
            label={loading ? t("inbox.refreshing") : t("inbox.refresh")}
            variant="secondary"
            onClick={() => {
              void refreshInbox({ force: true });
            }}
            disabled={loading}
          />
        </div>

        <section className="inbox-group">
          <h3>{t("inbox.section.repositories")}</h3>
          <div className="repo-selector">
            {repoEntries.map((entry) => (
              <label key={entry.key} className="repo-option">
                <input
                  type="checkbox"
                  checked={selectedRepoKeys.includes(entry.key)}
                  onChange={(event) => {
                    setSelectedRepoKeys((current) => {
                      if (event.target.checked) {
                        if (current.includes(entry.key)) {
                          return current;
                        }
                        return [...current, entry.key];
                      }
                      return current.filter((value) => value !== entry.key);
                    });
                  }}
                />
                <span>{entry.key}</span>
                <span className="tag">{entry.viewerPermission ?? t("common.unknown")}</span>
              </label>
            ))}
          </div>
        </section>

        <section className="inbox-group">
          <h3>{t("inbox.section.filters")}</h3>
          <label>
            <span>{t("inbox.filter.search")}</span>
            <input
              className="input"
              value={filters.query}
              onChange={(event) =>
                setFilters((current) => ({ ...current, query: event.target.value }))
              }
              placeholder={t("inbox.filter.search_placeholder")}
            />
          </label>
          <label>
            <span>{t("inbox.filter.state")}</span>
            <select
              className="input"
              value={filters.state}
              onChange={(event) =>
                setFilters((current) => ({
                  ...current,
                  state: event.target.value as InboxFilters["state"],
                }))
              }
            >
              <option value="all">{t("inbox.filter.state_all")}</option>
              <option value="open">{t("inbox.filter.state_open")}</option>
              <option value="closed">{t("inbox.filter.state_closed")}</option>
              {mode === "pr" ? (
                <option value="merged">{t("inbox.filter.state_merged")}</option>
              ) : null}
            </select>
          </label>
          <label>
            <span>{t("inbox.filter.label")}</span>
            <input
              className="input"
              value={filters.label}
              onChange={(event) =>
                setFilters((current) => ({ ...current, label: event.target.value }))
              }
              placeholder={t("inbox.filter.label_placeholder")}
            />
          </label>
          <label>
            <span>{t("inbox.filter.assignee")}</span>
            <input
              className="input"
              value={filters.assignee}
              onChange={(event) =>
                setFilters((current) => ({ ...current, assignee: event.target.value }))
              }
              placeholder={t("inbox.filter.assignee_placeholder")}
            />
          </label>
          {mode === "pr" ? (
            <label>
              <span>{t("inbox.filter.reviewer")}</span>
              <input
                className="input"
                value={filters.reviewer}
                onChange={(event) =>
                  setFilters((current) => ({ ...current, reviewer: event.target.value }))
                }
                placeholder={t("inbox.filter.reviewer_placeholder")}
              />
            </label>
          ) : null}
          {mode === "pr" ? (
            <label>
              <span>{t("inbox.filter.draft")}</span>
              <select
                className="input"
                value={filters.draft}
                onChange={(event) =>
                  setFilters((current) => ({
                    ...current,
                    draft: event.target.value as InboxFilters["draft"],
                  }))
                }
              >
                <option value="all">{t("inbox.filter.draft_all")}</option>
                <option value="yes">{t("inbox.filter.draft_yes")}</option>
                <option value="no">{t("inbox.filter.draft_no")}</option>
              </select>
            </label>
          ) : null}
          <label>
            <span>{t("inbox.filter.updated_within")}</span>
            <input
              className="input"
              value={filters.updatedWithinHours}
              onChange={(event) =>
                setFilters((current) => ({
                  ...current,
                  updatedWithinHours: event.target.value,
                }))
              }
              placeholder={t("inbox.filter.updated_within_placeholder")}
            />
          </label>
          <label>
            <span>{t("inbox.filter.sort")}</span>
            <select
              className="input"
              value={sortMode}
              onChange={(event) => setSortMode(event.target.value as SortMode)}
            >
              <option value="priority">{t("inbox.sort.priority")}</option>
              <option value="updated_desc">{t("inbox.sort.updated_desc")}</option>
              <option value="updated_asc">{t("inbox.sort.updated_asc")}</option>
            </select>
          </label>
          <label>
            <span>{t("inbox.filter.sla_hours")}</span>
            <input
              className="input"
              value={slaHours}
              onChange={(event) => setSlaHours(event.target.value)}
              placeholder={t("inbox.filter.sla_hours_placeholder")}
            />
          </label>
        </section>

        <section className="inbox-group">
          <h3>{t("inbox.section.saved_views")}</h3>
          <div className="row gap-sm">
            <IconButton
              icon={Save}
              label={t("inbox.saved.save_current")}
              variant="secondary"
              onClick={() => openModal({ kind: "save_view" })}
            />
            <IconButton
              icon={RotateCcw}
              label={t("inbox.saved.reset")}
              variant="secondary"
              onClick={() => {
                setFilters(initialInboxFilters());
                setSortMode("priority");
                setSelectedViewId("");
              }}
            />
          </div>

          <div className="saved-views">
            {allViews.length === 0 ? <p className="info-text">{t("inbox.saved.empty")}</p> : null}
            {allViews.map((view) => (
              <div
                key={view.id}
                className={view.id === selectedViewId ? "saved-view active" : "saved-view"}
              >
                <button
                  type="button"
                  className="btn secondary"
                  onClick={() => {
                    setSelectedViewId(view.id);
                    setSelectedRepoKeys(
                      view.repoKeys.length > 0
                        ? view.repoKeys
                        : repoEntries.map((entry) => entry.key),
                    );
                    setFilters(view.filters);
                    setSortMode(view.sortMode);
                  }}
                >
                  {view.name}
                </button>
                {!view.builtIn ? (
                  <IconButton
                    icon={Trash2}
                    label={t("inbox.saved.delete")}
                    variant="secondary"
                    onClick={() => {
                      setCustomViews((current) =>
                        current.filter((item) => item.id !== view.id),
                      );
                      if (selectedViewId === view.id) {
                        setSelectedViewId("");
                      }
                    }}
                  />
                ) : null}
              </div>
            ))}
          </div>
        </section>

        <section className="inbox-group">
          <h3>{t("inbox.section.activity")}</h3>
          <div className="activity-list">
            {recentActivity.map((entry) => (
              <div key={`${entry.requestId}-${entry.timestamp}`} className="activity-item">
                <span>{entry.commandId}</span>
                <span className={entry.status === "success" ? "badge success" : "badge danger"}>
                  {entry.status}
                </span>
              </div>
            ))}
          </div>
        </section>
      </aside>

      <section className="inbox-panel inbox-center">
        <header className="section-header">
          <h2>{t("inbox.queue.title")}</h2>
          <p>{fmt("inbox.queue.count", { count: filteredItems.length })}</p>
        </header>

        {loading ? <LoadingIndicator size="sm" label={t("inbox.loading")} /> : null}

        <div className="row gap-sm wrap">
          <IconButton
            icon={CircleX}
            label={t("inbox.batch.execute_selected")}
            variant="secondary"
            onClick={openBatchCloseModal}
            disabled={batchRunning || selectedBatchTargets.length === 0}
          />
          <IconButton
            icon={Check}
            label={t("inbox.batch.select_visible")}
            variant="secondary"
            onClick={() => {
              setCheckedItemIds((current) => {
                const selected = new Set(current);
                for (const item of filteredItems) {
                  selected.add(item.id);
                }
                return [...selected];
              });
            }}
            disabled={filteredItems.length === 0}
          />
          <IconButton
            icon={RotateCcw}
            label={t("inbox.batch.clear_selection")}
            variant="secondary"
            onClick={() => setCheckedItemIds([])}
            disabled={checkedItemIds.length === 0}
          />
          <IconButton
            icon={Eraser}
            label={t("inbox.batch.clear")}
            variant="secondary"
            onClick={() => setBatchResult(null)}
            disabled={!batchResult}
          />
        </div>

        {batchRunning && batchProgress ? (
          <LoadingIndicator
            size="sm"
            label={
              fmt("inbox.batch.progress", {
                processed: batchProgress.processed,
                total: batchProgress.total,
              })
            }
          />
        ) : null}

        {batchResult ? (
          <p className="info-text">
            {fmt("inbox.batch.result", {
              total: batchResult.total,
              success: batchResult.success,
              failed: batchResult.failed,
              skipped: batchResult.skipped,
            })}
          </p>
        ) : null}

        {error ? <p className="error-text">{error}</p> : null}
        {fetchWarnings.length > 0 ? (
          <div className="warn-text">
            <p>{fmt("inbox.warning.partial_fetch", { count: fetchWarnings.length })}</p>
            {fetchWarnings.slice(0, 3).map((message) => (
              <p key={message}>{message}</p>
            ))}
          </div>
        ) : null}
        <div className="queue-list">
          {filteredItems.map((item) => {
            const timing = deriveItemTiming(item.updatedAt, resolvedSlaHours);
            return (
              <article
                key={item.id}
                className={
                  detailOpen && item.id === selectedItem?.id ? "queue-item active" : "queue-item"
                }
                role="button"
                tabIndex={0}
                onClick={() => {
                  setSelectedItemId(item.id);
                  setDetailOpen(true);
                }}
                onKeyDown={(event) => {
                  if (event.key === "Enter" || event.key === " ") {
                    event.preventDefault();
                    setSelectedItemId(item.id);
                    setDetailOpen(true);
                  }
                }}
              >
                <div className="queue-item-header">
                  <label
                    className="queue-item-check"
                    onClick={(event) => event.stopPropagation()}
                  >
                    <input
                      type="checkbox"
                      checked={checkedItemIdSet.has(item.id)}
                      onKeyDown={(event) => event.stopPropagation()}
                      onChange={(event) => {
                        setCheckedItemIds((current) => {
                          if (event.target.checked) {
                            if (current.includes(item.id)) {
                              return current;
                            }
                            return [...current, item.id];
                          }
                          return current.filter((id) => id !== item.id);
                        });
                      }}
                      aria-label={`${item.kind.toUpperCase()} #${item.number}`}
                    />
                  </label>
                  <span className="tag">{item.kind.toUpperCase()}</span>
                  <span className="tag">
                    {item.owner}/{item.repo}
                  </span>
                  <span className="tag">#{item.number}</span>
                  <span className={timing.isStale ? "tag danger" : "tag"}>{item.state}</span>
                  {item.isDraft ? <span className="tag">{t("inbox.queue.badge.draft")}</span> : null}
                  {timing.isStale ? (
                    <span className="tag danger">{t("inbox.queue.badge.stale")}</span>
                  ) : null}
                </div>
                <strong>{item.title}</strong>
                <div className="queue-meta">
                  <span>
                    {fmt("inbox.queue.meta.author", {
                      author: item.author ?? t("common.unknown"),
                    })}
                  </span>
                  <span>{fmt("inbox.queue.meta.updated", { updated: formatUpdatedLabel(item.updatedAt) })}</span>
                  {item.reviewDecision ? (
                    <span>
                      {fmt("inbox.queue.meta.review", {
                        decision: item.reviewDecision,
                      })}
                    </span>
                  ) : null}
                </div>
                <div className="tag-row">
                  {item.labels.slice(0, 4).map((label) => (
                    <span className="tag" key={`${item.id}-label-${label}`}>
                      {label}
                    </span>
                  ))}
                  {item.assignees.slice(0, 3).map((assignee) => (
                    <span className="tag" key={`${item.id}-assignee-${assignee}`}>
                      @{assignee}
                    </span>
                  ))}
                  {item.reviewers.slice(0, 3).map((reviewer) => (
                    <span className="tag" key={`${item.id}-reviewer-${reviewer}`}>
                      review:{reviewer}
                    </span>
                  ))}
                </div>
              </article>
            );
          })}
        </div>
      </section>

      {detailOpen && selectedItem ? (
        <div
          className="inbox-detail-backdrop"
          role="presentation"
          onClick={() => setDetailOpen(false)}
        >
          <section
            className="inbox-panel inbox-detail-sheet"
            role="dialog"
            aria-modal="true"
            aria-label={`${selectedItem.kind.toUpperCase()} #${selectedItem.number}`}
            onClick={(event) => event.stopPropagation()}
          >
            <div className="row gap-sm wrap">
              <IconButton
                icon={ArrowLeft}
                label={t("inbox.detail.back")}
                variant="secondary"
                onClick={() => setDetailOpen(false)}
              />
            </div>
            <header className="section-header">
              <h2>
                {selectedItem.kind.toUpperCase()} #{selectedItem.number}
              </h2>
              <p>
                {selectedItem.owner}/{selectedItem.repo}
              </p>
            </header>

            <div className="row gap-sm wrap">
              <IconButton
                icon={ExternalLink}
                label={t("inbox.open_github")}
                variant="secondary"
                onClick={() => {
                  void openExternalUrl(selectedItem.url);
                }}
              />
              <IconButton
                icon={Undo2}
                label={t("inbox.action.reopen")}
                variant="secondary"
                disabled={Boolean(guardedReasonForSelected("reopen")) || mutationRunning}
                title={guardedReasonForSelected("reopen") ? actionGuardMessage(t, guardedReasonForSelected("reopen") as ActionGuardReason) : undefined}
                onClick={() => {
                  void reopenSelected();
                }}
              />
              <IconButton
                icon={CircleX}
                label={t("inbox.action.close_shortcut")}
                variant="danger"
                disabled={Boolean(guardedReasonForSelected("close")) || mutationRunning}
                title={guardedReasonForSelected("close") ? actionGuardMessage(t, guardedReasonForSelected("close") as ActionGuardReason) : undefined}
                onClick={() => {
                  void closeSelected();
                }}
              />
              <IconButton
                icon={MessageSquare}
                label={t("inbox.action.comment_shortcut")}
                variant="secondary"
                disabled={Boolean(guardedReasonForSelected("comment")) || mutationRunning}
                title={guardedReasonForSelected("comment") ? actionGuardMessage(t, guardedReasonForSelected("comment") as ActionGuardReason) : undefined}
                onClick={() => openModal({ kind: "comment", item: selectedItem })}
              />
            </div>

            {selectedItem.kind === "issue" ? (
              <>
                <section className="inbox-group">
                  <h3>{t("inbox.issue.quick_edit")}</h3>
                  <div className="row gap-sm wrap">
                    <IconButton
                      icon={UserPlus}
                      label={t("inbox.issue.add_assignees")}
                      variant="secondary"
                      disabled={Boolean(guardedReasonForSelected("issue_edit")) || mutationRunning}
                      onClick={() =>
                        openModal({
                          kind: "issue_edit",
                          item: selectedItem,
                          field: "add_assignees",
                        })
                      }
                    />
                    <IconButton
                      icon={UserMinus}
                      label={t("inbox.issue.remove_assignees")}
                      variant="secondary"
                      disabled={Boolean(guardedReasonForSelected("issue_edit")) || mutationRunning}
                      onClick={() =>
                        openModal({
                          kind: "issue_edit",
                          item: selectedItem,
                          field: "remove_assignees",
                        })
                      }
                    />
                    <IconButton
                      icon={Tag}
                      label={t("inbox.issue.add_labels")}
                      variant="secondary"
                      disabled={Boolean(guardedReasonForSelected("issue_edit")) || mutationRunning}
                      onClick={() =>
                        openModal({
                          kind: "issue_edit",
                          item: selectedItem,
                          field: "add_labels",
                        })
                      }
                    />
                    <IconButton
                      icon={Trash2}
                      label={t("inbox.issue.remove_labels")}
                      variant="secondary"
                      disabled={Boolean(guardedReasonForSelected("issue_edit")) || mutationRunning}
                      onClick={() =>
                        openModal({
                          kind: "issue_edit",
                          item: selectedItem,
                          field: "remove_labels",
                        })
                      }
                    />
                  </div>
                </section>

                <section className="inbox-group">
                  <h3>{t("inbox.issue.detail_title")}</h3>
                  {issueDetailState.loading ? (
                    <LoadingIndicator size="sm" label={t("inbox.issue.loading")} />
                  ) : null}
                  {issueDetailState.error ? (
                    <p className="error-text">{issueDetailState.error}</p>
                  ) : null}
                  {issueDetailState.detail ? (
                    <>
                      <div className="detail-grid">
                        <span>
                          {fmt("inbox.issue.detail_state", {
                            state: issueDetailState.detail.state,
                          })}
                        </span>
                        <span>
                          {fmt("inbox.queue.meta.updated", {
                            updated: formatUpdatedLabel(
                              issueDetailState.detail.updatedAt ?? selectedItem.updatedAt,
                            ),
                          })}
                        </span>
                      </div>
                      <div className="tag-row">
                        {(issueDetailState.detail.labels ?? [])
                          .map((entry) => asString(entry.name))
                          .filter((entry): entry is string => Boolean(entry))
                          .map((label) => (
                            <span className="tag" key={`issue-label-${label}`}>
                              {label}
                            </span>
                          ))}
                        {(issueDetailState.detail.assignees ?? [])
                          .map((entry) => asString(entry.login))
                          .filter((entry): entry is string => Boolean(entry))
                          .map((assignee) => (
                            <span className="tag" key={`issue-assignee-${assignee}`}>
                              @{assignee}
                            </span>
                          ))}
                      </div>
                      <article className="issue-body">
                        {issueDetailState.detail.body || t("common.not_available")}
                      </article>
                    </>
                  ) : null}
                </section>

                <section className="inbox-group">
                  <h3>{t("inbox.issue.comments")}</h3>
                  <div className="comment-list">
                    {(issueDetailState.detail?.comments ?? []).map((comment, index) => (
                      <article key={`${comment.id ?? "issue-comment"}-${index}`} className="comment-item">
                        <header>
                          <strong>{comment.author?.login ?? t("common.unknown")}</strong>
                          <span>{formatUpdatedLabel(comment.createdAt ?? null)}</span>
                        </header>
                        <p>{comment.body}</p>
                        <div className="row gap-sm wrap">
                          <IconButton
                            icon={MessageSquare}
                            label={t("inbox.issue.reply")}
                            variant="secondary"
                            disabled={Boolean(guardedReasonForSelected("comment")) || mutationRunning}
                            onClick={() =>
                              openModal({
                                kind: "comment",
                                item: selectedItem,
                                initialBody: buildIssueReplyTemplate(comment),
                              })
                            }
                          />
                          {comment.url ? (
                            <IconButton
                              icon={ExternalLink}
                              label={t("inbox.open_github")}
                              variant="secondary"
                              onClick={() => {
                                void openExternalUrl(comment.url ?? "");
                              }}
                            />
                          ) : null}
                        </div>
                      </article>
                    ))}
                    {(issueDetailState.detail?.comments ?? []).length === 0 && !issueDetailState.loading ? (
                      <p className="info-text">{t("inbox.issue.no_comments")}</p>
                    ) : null}
                  </div>
                </section>
              </>
            ) : null}

            {selectedItem.kind === "pr" ? (
              <>
                <section className="inbox-group">
                  <h3>{t("inbox.pr.quick_actions")}</h3>
                  <div className="row gap-sm wrap">
                    <IconButton
                      icon={Check}
                      label={t("inbox.pr.approve_shortcut")}
                      variant="primary"
                      disabled={Boolean(guardedReasonForSelected("approve")) || mutationRunning}
                      title={guardedReasonForSelected("approve") ? actionGuardMessage(t, guardedReasonForSelected("approve") as ActionGuardReason) : undefined}
                      onClick={() => openModal({ kind: "approve", item: selectedItem })}
                    />
                    <IconButton
                      icon={CircleX}
                      label={t("inbox.pr.request_changes")}
                      variant="secondary"
                      disabled={Boolean(guardedReasonForSelected("request_changes")) || mutationRunning}
                      title={guardedReasonForSelected("request_changes") ? actionGuardMessage(t, guardedReasonForSelected("request_changes") as ActionGuardReason) : undefined}
                      onClick={() => openModal({ kind: "request_changes", item: selectedItem })}
                    />
                    <IconButton
                      icon={GitMerge}
                      label={t("inbox.pr.merge")}
                      variant="secondary"
                      disabled={Boolean(guardedReasonForSelected("merge")) || mutationRunning}
                      title={guardedReasonForSelected("merge") ? actionGuardMessage(t, guardedReasonForSelected("merge") as ActionGuardReason) : undefined}
                      onClick={() => openModal({ kind: "merge", item: selectedItem })}
                    />
                    <IconButton
                      icon={Code2}
                      label={t("inbox.pr.detail_json")}
                      variant="secondary"
                      onClick={() => onInspect(t("inbox.pr.detail_payload"), prDetailState)}
                      disabled={prDetailState.loading}
                    />
                  </div>
                  {selectedPermission ? (
                    <p className="info-text">
                      {fmt("inbox.permission", {
                        permission: selectedPermission,
                      })}
                    </p>
                  ) : null}
                </section>

                <section className="inbox-group">
                  <h3>{t("inbox.pr.detail_title")}</h3>
                  {prDetailState.loading ? (
                    <LoadingIndicator size="sm" label={t("inbox.pr.loading")} />
                  ) : null}
                  {prDetailState.error ? <p className="error-text">{prDetailState.error}</p> : null}
                  {prDetailState.warning ? <p className="warn-text">{prDetailState.warning}</p> : null}
                  {prDetailState.detail ? (
                    <div className="detail-grid">
                      <span>{fmt("inbox.pr.detail_state", { state: prDetailState.detail.state })}</span>
                      <span>
                        {fmt("inbox.pr.detail_merge_state", {
                          state: prDetailState.detail.mergeStateStatus ?? "-",
                        })}
                      </span>
                      <span>
                        {fmt("inbox.pr.detail_review", {
                          decision: prDetailState.detail.reviewDecision ?? "-",
                        })}
                      </span>
                      <span>
                        {fmt("inbox.pr.detail_diff", {
                          additions: prDetailState.detail.additions ?? 0,
                          deletions: prDetailState.detail.deletions ?? 0,
                          files: prDetailState.detail.changedFiles ?? 0,
                        })}
                      </span>
                    </div>
                  ) : null}
                </section>

                <section className="inbox-group">
                  <h3>
                    {fmt("inbox.pr.unresolved_threads", {
                      count: unresolvedThreads.length,
                    })}
                  </h3>
                  <div className="thread-list">
                    {unresolvedThreads.map((thread) => (
                      <button
                        type="button"
                        key={thread.thread_id}
                        className="thread-item"
                        onClick={() => {
                          if (thread.path) {
                            setSelectedDiffPath(thread.path);
                          }
                        }}
                      >
                        <span>{thread.path ?? t("inbox.pr.thread_path_unknown")}</span>
                        <span>
                          {thread.line
                            ? fmt("inbox.pr.thread_line", { line: thread.line })
                            : t("inbox.pr.thread_line_unknown")}
                        </span>
                        <span>
                          {thread.is_outdated
                            ? t("inbox.pr.thread_outdated")
                            : t("inbox.pr.thread_active")}
                        </span>
                      </button>
                    ))}
                    {unresolvedThreads.length === 0 ? (
                      <p className="info-text">{t("inbox.pr.no_unresolved_threads")}</p>
                    ) : null}
                  </div>
                </section>

                <section className="inbox-group">
                  <h3>{t("inbox.pr.diff_viewer")}</h3>
                  <div className="row gap-sm wrap">
                    <IconButton
                      icon={AlignJustify}
                      label={t("inbox.pr.diff_mode.inline")}
                      variant={diffViewMode === "inline" ? "primary" : "secondary"}
                      onClick={() => setDiffViewMode("inline")}
                    />
                    <IconButton
                      icon={SplitSquareHorizontal}
                      label={t("inbox.pr.diff_mode.split")}
                      variant={diffViewMode === "split" ? "primary" : "secondary"}
                      onClick={() => setDiffViewMode("split")}
                    />
                  </div>
                  <div className="diff-layout">
                    <div className="diff-files">
                      {prDetailState.diffFiles.map((file) => (
                        <button
                          key={file.filename}
                          type="button"
                          className={selectedDiffPath === file.filename ? "diff-file active" : "diff-file"}
                          onClick={() => setSelectedDiffPath(file.filename)}
                        >
                          <span>{file.filename}</span>
                          <span>
                            +{file.additions} / -{file.deletions}
                          </span>
                        </button>
                      ))}
                    </div>
                    <Suspense fallback={<LoadingIndicator size="sm" label={t("inbox.loading")} />}>
                      <LazyDiffSyntaxPreview
                        content={diffPreviewText}
                        filename={selectedDiffFile?.filename ?? selectedDiffPath}
                        emptyLabel={t("inbox.pr.no_diff")}
                        viewMode={diffViewMode}
                      />
                    </Suspense>
                  </div>
                </section>

                <section className="inbox-group">
                  <h3>{t("inbox.pr.comments")}</h3>
                  <div className="comment-list">
                    {prDetailState.comments.slice(0, 20).map((comment, index) => (
                      <article key={`${comment.id ?? "c"}-${index}`} className="comment-item">
                        <header>
                          <strong>{comment.author?.login ?? t("common.unknown")}</strong>
                          <span>{formatUpdatedLabel(comment.created_at ?? null)}</span>
                        </header>
                        <p>{comment.body}</p>
                      </article>
                    ))}
                    {prDetailState.comments.length === 0 ? (
                      <p className="info-text">{t("inbox.pr.no_comments")}</p>
                    ) : null}
                  </div>
                </section>
              </>
            ) : null}
          </section>
        </div>
      ) : null}

      {modalConfig ? (
        <InboxActionModal
          open
          title={modalConfig.title}
          description={modalConfig.description}
          fields={modalConfig.fields}
          previewItems={modalConfig.previewItems}
          confirmLabel={modalConfig.confirmLabel}
          cancelLabel={t("common.cancel")}
          confirmToken={modalConfig.confirmToken}
          tokenHint={modalConfig.tokenHint}
          tokenLabel={modalConfig.tokenLabel}
          tokenPlaceholder={modalConfig.tokenPlaceholder}
          requiredFieldMessage={(label) =>
            fmt("inbox.modal.validation.required_field", { field: label })
          }
          tokenMismatchMessage={t("inbox.modal.validation.token_mismatch")}
          danger={modalConfig.danger}
          running={modalRunning}
          runningLabel={t("common.loading")}
          errorMessage={modalError}
          onCancel={closeModal}
          onConfirm={handleModalConfirm}
        />
      ) : null}
    </section>
  );
}

function normalizeIssueDetail(value: IssueDetail): IssueDetail {
  const record = asRecord(value);
  const commentsRaw = Array.isArray(record.comments) ? record.comments : [];
  const assigneesRaw = Array.isArray(record.assignees) ? record.assignees : [];
  const labelsRaw = Array.isArray(record.labels) ? record.labels : [];

  const comments: IssueCommentEntry[] = [];
  for (const entry of commentsRaw) {
    const item = asRecord(entry);
    const body = asString(item.body);
    if (!body) {
      continue;
    }

    const authorRecord = asRecord(item.author);
    const userRecord = asRecord(item.user);

    comments.push({
      id: asNumber(item.id) ?? asNumber(item.databaseId) ?? undefined,
      body,
      createdAt: asString(item.createdAt) ?? asString(item.created_at),
      author: {
        login:
          asString(authorRecord.login) ??
          asString(authorRecord.name) ??
          asString(userRecord.login) ??
          undefined,
      },
      url: asString(item.url) ?? asString(item.html_url),
    });
  }

  const assignees = assigneesRaw
    .map((entry) => {
      const login = asString(asRecord(entry).login);
      if (!login) {
        return null;
      }
      return { login };
    })
    .filter((entry): entry is { login: string } => entry !== null);

  const labels = labelsRaw
    .map((entry) => {
      const name = asString(asRecord(entry).name);
      if (!name) {
        return null;
      }
      return { name };
    })
    .filter((entry): entry is { name: string } => entry !== null);

  return {
    number: asNumber(record.number) ?? 0,
    title: asString(record.title) ?? "",
    state: asString(record.state) ?? "",
    url: asString(record.url) ?? "",
    body: asString(record.body) ?? "",
    comments,
    assignees,
    labels,
    updatedAt: asString(record.updatedAt) ?? asString(record.updated_at),
  };
}

function buildIssueReplyTemplate(comment: IssueCommentEntry): string {
  const mentionLogin = asString(asRecord(comment.author).login);
  const mentionLine = mentionLogin ? `@${mentionLogin}` : "";
  const quotedBody = comment.body
    .split("\n")
    .map((line) => `> ${line}`)
    .join("\n");

  return [mentionLine, quotedBody, ""].filter((line) => line.length > 0).join("\n\n");
}

function dedupeStrings(values: string[]): string[] {
  const deduped: string[] = [];
  for (const value of values) {
    const trimmed = value.trim();
    if (!trimmed) {
      continue;
    }
    if (!deduped.includes(trimmed)) {
      deduped.push(trimmed);
    }
  }
  return deduped;
}

function mapPullRequests(owner: string, repo: string, rows: unknown[]): InboxItem[] {
  if (!Array.isArray(rows)) {
    return [];
  }

  return rows
    .map((row): InboxItem | null => {
      const record = asRecord(row);
      const number = asNumber(record.number);
      const title = asString(record.title);
      const url = asString(record.url);
      if (!number || !title || !url) {
        return null;
      }

      return {
        id: `pr:${owner}/${repo}#${number}`,
        kind: "pr",
        owner,
        repo,
        number,
        title,
        state: asString(record.state) ?? "OPEN",
        url,
        author: asString(asRecord(record.author).login),
        labels: extractNamedValues(record.labels, "name"),
        assignees: extractNamedValues(record.assignees, "login"),
        reviewers: extractReviewerLogins(record.reviewRequests),
        isDraft: Boolean(record.isDraft),
        reviewDecision: asString(record.reviewDecision),
        updatedAt: asString(record.updatedAt),
      };
    })
    .filter((item): item is InboxItem => item !== null);
}

function mapIssues(owner: string, repo: string, rows: unknown[]): InboxItem[] {
  if (!Array.isArray(rows)) {
    return [];
  }

  return rows
    .map((row): InboxItem | null => {
      const record = asRecord(row);
      const number = asNumber(record.number);
      const title = asString(record.title);
      const url = asString(record.url);
      if (!number || !title || !url) {
        return null;
      }

      return {
        id: `issue:${owner}/${repo}#${number}`,
        kind: "issue",
        owner,
        repo,
        number,
        title,
        state: asString(record.state) ?? "OPEN",
        url,
        author: asString(asRecord(record.author).login),
        labels: extractNamedValues(record.labels, "name"),
        assignees: extractNamedValues(record.assignees, "login"),
        reviewers: [],
        isDraft: false,
        reviewDecision: null,
        updatedAt: asString(record.updatedAt),
      };
    })
    .filter((item): item is InboxItem => item !== null);
}

function buildDefaultViews(t: (key: string) => string, mode: ItemKind): SavedView[] {
  const views: SavedView[] = [];

  if (mode === "pr") {
    views.push({
      id: "default-review-waiting",
      name: t("inbox.saved.default.review_waiting"),
      repoKeys: [],
      filters: {
        ...initialInboxFilters(),
        state: "open",
        reviewer: "@me",
        draft: "no",
      },
      sortMode: "priority",
      builtIn: true,
    });
  }

  views.push(
    {
      id: "default-stale",
      name: t("inbox.saved.default.stale"),
      repoKeys: [],
      filters: {
        ...initialInboxFilters(),
        state: "open",
      },
      sortMode: "priority",
      builtIn: true,
    },
    {
      id: "default-mine",
      name: t("inbox.saved.default.mine"),
      repoKeys: [],
      filters: {
        ...initialInboxFilters(),
        state: "open",
        assignee: "@me",
      },
      sortMode: "updated_desc",
      builtIn: true,
    },
  );

  return views;
}

function getSavedViewsStorageKey(mode: ItemKind): string {
  return mode === "pr"
    ? "gh-client-pr-custom-views"
    : "gh-client-issue-custom-views";
}

function loadCustomViews(storageKey: string): SavedView[] {
  try {
    const raw = localStorage.getItem(storageKey);
    if (!raw) {
      return [];
    }

    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) {
      return [];
    }

    const views: SavedView[] = [];
    for (const entry of parsed) {
      const record = asRecord(entry);
      const id = asString(record.id);
      const name = asString(record.name);
      const sortModeRaw = asString(record.sortMode);
      const sortMode =
        sortModeRaw === "priority" || sortModeRaw === "updated_desc" || sortModeRaw === "updated_asc"
          ? sortModeRaw
          : "priority";

      if (!id || !name) {
        continue;
      }

      const repoKeys = Array.isArray(record.repoKeys)
        ? record.repoKeys
            .map((value) => asString(value))
            .filter((value): value is string => Boolean(value))
        : [];

      const filtersRecord = asRecord(record.filters);
      const filters: InboxFilters = {
        query: asString(filtersRecord.query) ?? "",
        state: parseStateFilter(asString(filtersRecord.state)),
        label: asString(filtersRecord.label) ?? "",
        assignee: asString(filtersRecord.assignee) ?? "",
        reviewer: asString(filtersRecord.reviewer) ?? "",
        draft: parseDraftFilter(asString(filtersRecord.draft)),
        updatedWithinHours: asString(filtersRecord.updatedWithinHours) ?? "",
      };

      views.push({
        id,
        name,
        repoKeys,
        filters,
        sortMode,
        builtIn: false,
      });
    }

    return views;
  } catch (_cause) {
    return [];
  }
}

function parseStateFilter(value: string | null): InboxFilters["state"] {
  if (value === "open" || value === "closed" || value === "merged" || value === "all") {
    return value;
  }

  return "all";
}

function parseDraftFilter(value: string | null): InboxFilters["draft"] {
  if (value === "yes" || value === "no" || value === "all") {
    return value;
  }

  return "all";
}

function buildModalConfig(
  modalState: ModalState,
  t: (key: string) => string,
  fmt: (key: string, vars: Record<string, string | number>) => string,
  selectedRepoKeys: string[],
  filters: InboxFilters,
  sortMode: SortMode,
  issueEditAssigneeOptions: string[],
): {
  title: string;
  description?: string;
  fields?: InboxActionModalField[];
  previewItems?: string[];
  confirmLabel: string;
  danger?: boolean;
  confirmToken?: string;
  tokenHint?: string;
  tokenLabel?: string;
  tokenPlaceholder?: string;
} | null {
  switch (modalState.kind) {
    case "none":
      return null;
    case "save_view":
      return {
        title: t("inbox.modal.save_view.title"),
        description: fmt("inbox.modal.save_view.description", {
          repos: selectedRepoKeys.length,
          state: filters.state,
          sort: sortMode,
        }),
        fields: [
          {
            name: "name",
            label: t("inbox.modal.save_view.name"),
            type: "text",
            required: true,
            placeholder: t("inbox.modal.save_view.placeholder"),
          },
        ],
        confirmLabel: t("inbox.modal.save_view.confirm"),
      };
    case "approve":
      return {
        title: t("inbox.modal.approve.title"),
        description: fmt("inbox.modal.approve.description", {
          repo: `${modalState.item.owner}/${modalState.item.repo}`,
          number: modalState.item.number,
        }),
        confirmLabel: t("inbox.modal.approve.confirm"),
      };
    case "comment":
      return {
        title: t("inbox.modal.comment.title"),
        description: fmt("inbox.modal.comment.description", {
          kind: modalState.item.kind.toUpperCase(),
          number: modalState.item.number,
        }),
        fields: [
          {
            name: "body",
            label: t("inbox.modal.comment.body"),
            type: "textarea",
            required: true,
            placeholder: t("inbox.modal.comment.placeholder"),
            initialValue: modalState.initialBody ?? "",
          },
        ],
        confirmLabel: t("inbox.modal.comment.confirm"),
      };
    case "request_changes":
      return {
        title: t("inbox.modal.request_changes.title"),
        description: fmt("inbox.modal.request_changes.description", {
          number: modalState.item.number,
        }),
        fields: [
          {
            name: "body",
            label: t("inbox.modal.request_changes.body"),
            type: "textarea",
            required: true,
            initialValue: t("inbox.modal.request_changes.default_body"),
          },
        ],
        confirmLabel: t("inbox.modal.request_changes.confirm"),
      };
    case "merge":
      return {
        title: t("inbox.modal.merge.title"),
        description: fmt("inbox.modal.merge.description", {
          number: modalState.item.number,
        }),
        fields: [
          {
            name: "method",
            label: t("inbox.modal.merge.method"),
            type: "select",
            options: [
              { label: t("inbox.modal.merge.method.squash"), value: "squash" },
              { label: t("inbox.modal.merge.method.merge"), value: "merge" },
              { label: t("inbox.modal.merge.method.rebase"), value: "rebase" },
            ],
            initialValue: "squash",
          },
        ],
        confirmLabel: t("inbox.modal.merge.confirm"),
      };
    case "issue_edit":
      if (
        (modalState.field === "add_assignees" || modalState.field === "remove_assignees") &&
        issueEditAssigneeOptions.length > 0
      ) {
        return {
          title: t(`inbox.modal.issue_edit.${modalState.field}.title`),
          description: fmt("inbox.modal.issue_edit.description", {
            number: modalState.item.number,
          }),
          fields: [
            {
              name: "selected_values",
              label: t("inbox.modal.issue_edit.select_values"),
              type: "select",
              multiple: true,
              options: issueEditAssigneeOptions.map((value) => ({
                label: value,
                value,
              })),
            },
            {
              name: "manual_values",
              label: t("inbox.modal.issue_edit.manual_values"),
              type: "text",
              placeholder: t("inbox.modal.issue_edit.placeholder"),
            },
          ],
          confirmLabel: t("inbox.modal.issue_edit.confirm"),
        };
      }

      return {
        title: t(`inbox.modal.issue_edit.${modalState.field}.title`),
        description: fmt("inbox.modal.issue_edit.description", {
          number: modalState.item.number,
        }),
        fields: [
          {
            name: "values",
            label: t("inbox.modal.issue_edit.values"),
            type: "textarea",
            required: true,
            placeholder: t("inbox.modal.issue_edit.placeholder"),
          },
        ],
        confirmLabel: t("inbox.modal.issue_edit.confirm"),
      };
    case "batch_close": {
      const previewItems = modalState.targets
        .slice(0, BATCH_PREVIEW_LIMIT)
        .map((item) => `${item.kind.toUpperCase()} ${item.owner}/${item.repo}#${item.number}`);
      const token = `BATCH:CLOSE:${modalState.targets.length}`;
      const hasIssueTargets = modalState.targets.some((item) => item.kind === "issue");
      return {
        title: t("inbox.modal.batch_close.title"),
        description: fmt("inbox.modal.batch_close.description", {
          count: modalState.targets.length,
        }),
        fields: hasIssueTargets
          ? [
              {
                name: "issue_reason",
                label: t("inbox.modal.batch_close.issue_reason"),
                type: "select",
                options: [
                  {
                    label: t("inbox.modal.batch_close.issue_reason.completed"),
                    value: "completed",
                  },
                  {
                    label: t("inbox.modal.batch_close.issue_reason.not_planned"),
                    value: "not planned",
                  },
                ],
                initialValue: "completed",
              },
            ]
          : undefined,
        previewItems,
        confirmLabel: t("inbox.modal.batch_close.confirm"),
        danger: true,
        confirmToken: token,
        tokenHint: t("inbox.modal.batch_close.token_hint"),
        tokenLabel: t("inbox.modal.batch_close.token_label"),
        tokenPlaceholder: t("inbox.modal.batch_close.token_placeholder"),
      };
    }
    default:
      return null;
  }
}

function actionGuardMessage(
  t: (key: string) => string,
  reason: ActionGuardReason,
): string {
  return t(`inbox.guard.${reason}`);
}

function formatTemplate(
  template: string,
  vars: Record<string, string | number>,
): string {
  return Object.entries(vars).reduce((acc, [key, value]) => {
    return acc.replaceAll(`{${key}}`, String(value));
  }, template);
}

function toFrontendInvokeError(cause: unknown, commandId: CommandId): FrontendInvokeError {
  if (cause instanceof ExecutionError) {
    return cause.detail;
  }

  return {
    code: "execution_error",
    message: toErrorMessage(cause),
    retryable: false,
    fingerprint: "inbox",
    request_id: "",
    command_id: commandId,
  };
}

function toErrorMessage(cause: unknown): string {
  if (cause instanceof Error) {
    return cause.message;
  }

  return String(cause);
}

function formatErrorForRepo(commandId: string, cause: unknown): string {
  return `${commandId}: ${toErrorMessage(cause)}`;
}

async function mapWithConcurrency<T, R>(
  values: T[],
  concurrency: number,
  mapper: (value: T) => Promise<R>,
): Promise<R[]> {
  if (values.length === 0) {
    return [];
  }

  const workers = Math.max(1, Math.min(concurrency, values.length));
  const results: R[] = new Array(values.length);
  let cursor = 0;

  const run = async () => {
    while (true) {
      const index = cursor;
      cursor += 1;
      if (index >= values.length) {
        return;
      }

      results[index] = await mapper(values[index]);
    }
  };

  await Promise.all(Array.from({ length: workers }, () => run()));
  return results;
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
