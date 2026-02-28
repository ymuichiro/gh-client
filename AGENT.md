# AGENT.md

このファイルは、このリポジトリを継続して編集するための前提知識と運用ルールをまとめる。

## 最優先方針
- Backend First: バックエンドが完成するまでフロントエンドは後付けにする。
- Feature-Based: 機能単位で `command/service/dto/tests` を完結させる。
- Test Realism First: テストは本番相当の実操作を優先する。
- Safety Exception: 破壊系はテスト時のみダミー化する。

## 必須ルール
- 作業履歴と意図は常に `docs/memory/` に追記する。
- 重大な設計判断、失敗、仕様差分の発見は必ず記録する。
- 次アクションが変わる変更をしたら `TODO.md` と `ROADMAP.md` を更新する。

## 現在の実装スナップショット（2026-02-28）
- Rust workspace を構成済み（root workspace + `src-tauri` crate）。
- `core` 実装済み:
  - `CommandRegistry`
  - `CommandExecutor`（`SAFE_TEST_MODE` で destructive no-op）
  - `PolicyGuard`
  - `TraceContext` / `AuditEvent`
  - `AppError` / `ErrorCode`
- `features/repositories` 実装済み:
  - list/create/delete service
  - command handler
  - DTO parse
  - unit tests
  - live test (`gh repo list`)
- `features/pull_requests` 実装済み:
  - list/create/review/merge service
  - command handler
  - DTO parse
  - unit tests
  - live test (`gh pr list`)
- `features/issues` 実装済み:
  - list/create/comment/close/reopen service
  - command handler
  - DTO parse
  - unit tests
  - live test (`gh issue list`)
- `features/actions` 実装済み:
  - workflow list / run list / rerun / cancel service
  - command handler
  - DTO parse
  - unit tests
  - live test (`gh workflow list`, `gh run list`)
- `features/releases` 実装済み:
  - list/create/delete service
  - command handler
  - DTO parse
  - unit tests
  - live test (`gh release list`)
- `features/settings` 実装済み:
  - collaborators list/add/remove service
  - command handler
  - DTO parse
  - unit tests
  - live test (`gh api repos/{owner}/{repo}/collaborators`)

## 既知の仕様メモ
- `gh repo list` は `--owner` フラグではなく owner を位置引数で渡す。
- `gh workflow list --json` は workflow がない repo で空文字を返す場合がある。
- live test は環境変数で制御する。

## テスト実行コマンド
- 全テスト:
  - `cargo test`
- 実操作テスト（全 feature list 系）:
  - `GH_TEST_OWNER=$(gh api user --jq .login) GH_TEST_REPO=$(gh repo list $(gh api user --jq .login) --json name --limit 1 --jq '.[0].name') GH_CLIENT_LIVE_TEST=1 cargo test --test repositories_live --test pull_requests_live --test issues_live --test actions_live --test releases_live --test settings_live -- --nocapture`

## 次の実装順序
1. cross-feature E2E の整備
2. security CI（`cargo audit`, `cargo deny`）
3. frontend attach

## 記録ファイル
- 履歴: `docs/memory/YYYY-MM-DD-*.md`
- TODO: `TODO.md`
- 中期計画: `ROADMAP.md`
