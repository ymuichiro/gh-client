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

## 現在の実装スナップショット（2026-03-01）
- Rust workspace を構成済み（root workspace + `src-tauri` crate）。
- `core` 実装済み:
  - `CommandRegistry`
  - `CommandExecutor`（`SAFE_TEST_MODE` で destructive no-op）
  - `PolicyGuard`
  - `TraceContext` / `AuditEvent`
  - `AppError` / `ErrorCode`
- `features/repositories` 実装済み:
  - list/create/edit/delete service
  - branch list/create/delete
  - commit list
  - command handler
  - DTO parse
  - unit tests
  - live test (`gh repo list`, branch/commit list via E2E)
- `features/pull_requests` 実装済み:
  - list/create/edit/close/reopen/review/merge service
  - command handler
  - DTO parse
  - unit tests
  - live test (`gh pr list`)
- `features/issues` 実装済み:
  - list/create/edit/comment/close/reopen service
  - command handler
  - DTO parse
  - unit tests
  - live test (`gh issue list`)
- `features/actions` 実装済み:
  - workflow list / run list / run detail / run logs / rerun / cancel service
  - command handler
  - DTO parse
  - unit tests
  - live test (`gh workflow list`, `gh run list`, run detail/logs via E2E)
- `features/releases` 実装済み:
  - list/create/edit/delete service
  - asset upload/delete
  - command handler
  - DTO parse
  - unit tests
  - live test (`gh release list`)
- `features/settings` 実装済み:
  - collaborators list/add/remove
  - secrets list/set/delete
  - variables list/set/delete
  - webhooks list/create/ping/delete
  - branch protection get/update（update は get 結果を併合して PUT）
  - deploy keys list/add/delete
  - dependabot alerts list
  - command handler
  - DTO parse
  - unit tests
  - live test（collaborators/secrets/variables/webhooks/deploy keys の read 系）
- cross-feature E2E 実装済み:
  - `repositories` / `pull_requests` / `issues` / `actions` / `releases` / `settings` の read-only 実操作を 1 本で検証

## 既知の仕様メモ
- `gh repo list` は `--owner` フラグではなく owner を位置引数で渡す。
- `gh workflow list --json` は workflow がない repo で空文字を返す場合がある。
- `gh pr create --json` は利用できないため、`gh api repos/{owner}/{repo}/pulls` を使用する。
- destructive command は `CommandRegistry` の `CommandSafety::Destructive` で定義し、`SAFE_TEST_MODE=true` 時に no-op となる。
- live test は環境変数で制御する。

## テスト実行コマンド
- 全テスト:
  - `cargo test`
- 実操作テスト（feature live + cross-feature E2E）:
  - `OWNER=$(gh api user --jq .login) REPO=$(gh repo list "$OWNER" --json name --limit 1 --jq '.[0].name') GH_CLIENT_LIVE_TEST=1 GH_TEST_OWNER="$OWNER" GH_TEST_REPO="$REPO" cargo test --test repositories_live --test pull_requests_live --test issues_live --test actions_live --test releases_live --test settings_live --test e2e_live -- --nocapture`

## 次の実装順序
1. security CI（`cargo audit`, `cargo deny`）
2. frontend attach（backend contract を薄く接続）

## 記録ファイル
- 履歴: `docs/memory/YYYY-MM-DD-*.md`
- TODO: `TODO.md`
- 中期計画: `ROADMAP.md`
