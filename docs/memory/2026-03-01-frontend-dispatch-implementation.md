# 2026-03-01 Frontend Dispatcher Implementation Log

## 意図
- `FrontendCommandEnvelope` を backend の唯一の実行入口として固定し、フロント実装時の payload/権限/レスポンス仕様の揺れを止める。
- 既存 feature service 群を再利用し、gh CLI wrapper 方針とエラー分類を維持する。

## 実装
- `src-tauri/src/frontend/mod.rs`
  - `FrontendDispatcher<R: Runner + Clone>` を追加。
  - `supported_command_ids()` で stable command surface を公開。
  - `execute_envelope()` で envelope validation + command routing を実装。
  - permission 未指定時は `Viewer` にフォールバック。
  - low-level command (`repo.topics.replace`, `repo.branch.ref.get`) 向け raw 実行パスを追加。
  - レスポンスは JSON DTO または `{ "ok": true }` に統一。
- `src-tauri/src/lib.rs`
  - `pub mod frontend;` を追加して公開。
- `src-tauri/src/core/executor.rs`
  - dispatcher で service を安全に再構築できるよう `ProcessRunner` に `Clone` を付与。

## テスト
- 追加/更新した dispatcher 単体テスト:
  - command surface と `STABLE_COMMAND_IDS` の一致
  - `auth.status` routing
  - `pr.comments.list` routing
  - permission 未指定時の deny (`settings.collaborators.list`)
  - payload validation error
  - raw command 実行 (`repo.branch.ref.get`)
- 実行コマンド:
  - `cargo fmt --all`
  - `cargo test`
- 結果:
  - unit/live/doc test 全件 pass（164 unit + live suites）

## 影響
- バックエンドは frontend から 1 つの envelope 契約で呼び出せる状態になった。
- 次段の GUI 実装は dispatcher 契約に従って進められる。
