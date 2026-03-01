# 2026-03-01 Frontend Full Attach (Tauri + React)

## 背景
- backend は `FrontendDispatcher` まで実装済みで、GUI 接続と E2E 自動化が未完了だった。
- 本作業で `payload contract v2` の command surface を GUI から網羅実行できる状態を実装した。

## 実装内容
1. Tauri 実行基盤
- `src-tauri/Cargo.toml`
  - `desktop` feature を追加
  - `gh-client-desktop` binary（feature required）
  - `gh-client-envelope-cli` binary（bridge 用）
- `src-tauri/src/desktop/mod.rs`
  - `frontend_execute` (`tauri::command`) を追加
  - `FrontendDispatcher<ProcessRunner>` を AppState で管理
- `src-tauri/src/app_ipc.rs`
  - `execute_frontend_envelope` と `FrontendInvokeError` を追加
  - `AppError -> invoke error` の安定変換
- `src-tauri/src/main.rs`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`

2. Frontend 実装（`ui/`）
- Vite + React + TypeScript を追加
- 主要画面:
  - Dashboard
  - Repositories
  - Pull Requests
  - Issues
  - Actions
  - Releases
  - Settings
  - P2 Coverage
  - Command Console
  - History
- `ui/src/core/commandCatalog.ts`
  - 全 `STABLE_COMMAND_IDS` の catalog を定義
  - `payloadSchema` / `responseSchema` / permission / exposure / destructive を保持
- `ui/src/core/executor.ts`
  - `executeCommand` を唯一入口化
  - mode: `tauri | mock | bridge`
- 破壊系 2段階確認モーダル、JSON drawer、履歴記録（localStorage）
- 日英切替（JA/EN）

3. E2E 基盤
- `@playwright/test` を導入
- mock read suite: `ui/e2e/read.spec.ts`
- live suite: `ui/e2e/live.spec.ts`
  - read 常時
  - write は `GH_CLIENT_LIVE_WRITE_TEST=1` で opt-in
- `ui/scripts/bridge-server.mjs`
  - browser -> local bridge -> `gh-client-envelope-cli` 実行
  - CORS 対応

## テスト
- Rust:
  - `cargo test` ✅
- Frontend unit/integration:
  - `npm --prefix ui run test` ✅
- Frontend build:
  - `npm --prefix ui run build` ✅
- Playwright mock E2E:
  - `npm --prefix ui run e2e` ✅
- Playwright live read:
  - `GH_CLIENT_LIVE_TEST=1 GH_TEST_OWNER=... GH_TEST_REPO=... npm --prefix ui run e2e:live` ✅
- Playwright live write opt-in:
  - `GH_CLIENT_LIVE_TEST=1 GH_CLIENT_LIVE_WRITE_TEST=1 GH_TEST_OWNER=... GH_TEST_REPO=... npm --prefix ui run e2e:write` ✅

## 備考
- `projects.list` は `read:project` scope 不足時に失敗しうるため、live E2E では optional error 許容として扱う。
- 認証は継続して `gh auth login` セッション依存。アプリ token は保持しない。
