# 2026-02-28 Backend Feature Completion

## 目的
- Backend First 方針に従い、主要 feature module（issues/actions/releases/settings）を一気に実装して、実運用可能なバックエンド状態に到達する。

## 実装内容
- `features/issues` を追加:
  - list/create/comment/close/reopen
  - DTO parse
  - command handler
  - unit tests
- `features/actions` を追加:
  - workflow list
  - run list / rerun / cancel
  - DTO parse
  - command handler
  - unit tests
- `features/releases` を追加:
  - list/create/delete
  - delete は `SAFE_TEST_MODE` で no-op
  - DTO parse
  - command handler
  - unit tests
- `features/settings` を追加:
  - collaborators list/add/remove
  - remove は `SAFE_TEST_MODE` で no-op
  - DTO parse
  - command handler
  - unit tests

## CommandRegistry 変更
- PR 作成を `gh pr create` から `gh api POST /repos/{owner}/{repo}/pulls` に変更。
  - 理由: `gh pr create` は `--json` 非対応のため。
- 追加 command_id:
  - `issue.*`
  - `workflow.list`, `run.*`
  - `release.*`
  - `settings.collaborators.*`

## 実装中に見つかった仕様差分
- `gh workflow list --json ...` は、workflow が存在しない repo で空文字を返すケースがある。
- 対応として、actions DTO parser は empty payload を空配列として扱う。

## 検証
- `cargo fmt --all`
- `cargo test`（unit/integration）
- 実操作テスト（本番相当）
  - `GH_CLIENT_LIVE_TEST=1 GH_TEST_OWNER=<owner> GH_TEST_REPO=<repo> cargo test --test repositories_live --test pull_requests_live --test issues_live --test actions_live --test releases_live --test settings_live -- --nocapture`

## 結果
- 全 unit test 成功
- 全 live test 成功（repo/pr/issues/actions/releases/settings の list 系）
- destructive 操作は safe mode で実行抑止されることを確認

## 次アクション
1. cross-feature E2E（連続操作シナリオ）を追加。
2. CI に `cargo audit` / `cargo deny` を組み込み。
3. フロントエンド接続前に API 契約を固定化（command payload schema を明文化）。
