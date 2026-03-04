import { createContext, useContext, useMemo, useState, type PropsWithChildren } from "react";

export type Language = "ja" | "en";

type Dictionary = Record<string, string>;

const dictionaries: Record<Language, Dictionary> = {
  ja: {
    "app.title": "gh-client",
    "app.subtitle": "gh CLI thin-wrapper GUI",
    "nav.dashboard": "ダッシュボード",
    "nav.repositories": "Repositories",
    "nav.pull_requests": "Pull Requests",
    "nav.issues": "Issues",
    "nav.actions": "Actions",
    "nav.releases": "Releases",
    "nav.settings": "Settings",
    "nav.p2": "P2 Coverage",
    "nav.console": "Command Console",
    "nav.history": "操作履歴",
    "header.owner": "Owner",
    "header.repo": "Repo",
    "header.permission": "Permission",
    "header.auth": "認証状態",
    "context.refresh": "候補を更新",
    "context.loading": "候補を取得中...",
    "context.no_owner_options": "owner がありません",
    "context.no_repo_options": "repo がありません",
    "auth.logged_in": "ログイン済み",
    "auth.logged_out": "未ログイン",
    "common.execute": "実行",
    "common.cancel": "キャンセル",
    "common.close": "閉じる",
    "common.response": "レスポンス",
    "common.payload": "ペイロード",
    "common.loading": "実行中...",
    "common.not_available": "未対応",
    "common.required": "必須",
    "common.optional": "任意",
    "common.error": "エラー",
    "common.success": "成功",
    "common.select_command": "コマンドを選択",
    "common.raw_json": "Raw JSON Payload",
    "common.gh_login_hint": "未ログインの場合は `gh auth login` を実行してください。",
    "console.title": "Command Console",
    "console.description": "全 command を schema 駆動フォームまたは raw JSON で実行します。",
    "history.title": "監査用ローカル履歴",
    "history.empty": "履歴はまだありません。",
    "confirm.title": "破壊的操作の確認",
    "confirm.step1": "この操作は破壊的です。続行する前に再確認してください。",
    "confirm.step2": "2段階確認: 以下を入力して実行を有効化",
    "confirm.placeholder": "CONFIRM",
    "confirm.invalid": "確認文字列が一致しません。",
    "status.permission_missing": "権限または repo コンテキストが不足しています。",
    "status.cancel_unavailable": "現在の操作はキャンセル不可です。",
    "p2.description": "Projects / Discussions / Wiki / Pages / Rulesets / Insights",
    "dashboard.description": "auth.status と repo.list を中心に、実行起点を提供します。"
  },
  en: {
    "app.title": "gh-client",
    "app.subtitle": "gh CLI thin-wrapper GUI",
    "nav.dashboard": "Dashboard",
    "nav.repositories": "Repositories",
    "nav.pull_requests": "Pull Requests",
    "nav.issues": "Issues",
    "nav.actions": "Actions",
    "nav.releases": "Releases",
    "nav.settings": "Settings",
    "nav.p2": "P2 Coverage",
    "nav.console": "Command Console",
    "nav.history": "History",
    "header.owner": "Owner",
    "header.repo": "Repo",
    "header.permission": "Permission",
    "header.auth": "Auth",
    "context.refresh": "Refresh options",
    "context.loading": "Loading options...",
    "context.no_owner_options": "No owners",
    "context.no_repo_options": "No repositories",
    "auth.logged_in": "Logged in",
    "auth.logged_out": "Logged out",
    "common.execute": "Execute",
    "common.cancel": "Cancel",
    "common.close": "Close",
    "common.response": "Response",
    "common.payload": "Payload",
    "common.loading": "Running...",
    "common.not_available": "N/A",
    "common.required": "Required",
    "common.optional": "Optional",
    "common.error": "Error",
    "common.success": "Success",
    "common.select_command": "Select command",
    "common.raw_json": "Raw JSON Payload",
    "common.gh_login_hint": "If not logged in, run `gh auth login`.",
    "console.title": "Command Console",
    "console.description": "Execute all commands via schema-driven forms or raw JSON payload.",
    "history.title": "Local Audit History",
    "history.empty": "No execution history yet.",
    "confirm.title": "Destructive action confirmation",
    "confirm.step1": "This is a destructive action. Confirm before execution.",
    "confirm.step2": "Step 2: type the token below to enable execution",
    "confirm.placeholder": "CONFIRM",
    "confirm.invalid": "Confirmation token does not match.",
    "status.permission_missing": "Missing permission or repo context.",
    "status.cancel_unavailable": "Cancellation is not available for this operation.",
    "p2.description": "Projects / Discussions / Wiki / Pages / Rulesets / Insights",
    "dashboard.description": "Execution entrypoint centered on auth.status and repo.list."
  },
};

interface I18nContextValue {
  language: Language;
  setLanguage: (lang: Language) => void;
  t: (key: string) => string;
}

const I18nContext = createContext<I18nContextValue | null>(null);

export function I18nProvider({ children }: PropsWithChildren): JSX.Element {
  const [language, setLanguage] = useState<Language>("ja");

  const value = useMemo<I18nContextValue>(
    () => ({
      language,
      setLanguage,
      t: (key: string) => dictionaries[language][key] ?? key,
    }),
    [language],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18nContextValue {
  const context = useContext(I18nContext);
  if (!context) {
    throw new Error("useI18n must be used inside I18nProvider");
  }
  return context;
}
