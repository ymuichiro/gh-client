# 2026-02-28 Pull Requests Feature

## 目的
- Backend First 計画に沿って、`features/pull_requests` を実装開始する。
- repositories と同じ構成で、feature-based に機能とテストを完結させる。

## 変更内容
- `src-tauri/src/features/pull_requests` を追加:
  - `command.rs`
  - `dto.rs`
  - `service.rs`
  - `mod.rs`
- `features/mod.rs` に `pull_requests` を追加。
- `CommandRegistry` に以下コマンドを追加:
  - `pr.list`
  - `pr.create`
  - `pr.review`
  - `pr.merge`
- live test を追加:
  - `src-tauri/tests/pull_requests_live.rs`

## 設計意図
- PR 操作の主要導線（list/create/review/merge）を backend で先行実装。
- 権限は `RepoPermission::Write` を要求し、service 境界で強制。
- DTO parse を分離し、`gh --json` 出力の変換失敗を `UpstreamError` で統一。

## 実施した検証
- `cargo fmt --all`
- `cargo test`
- 実操作テスト:
  - `GH_CLIENT_LIVE_TEST=1 GH_TEST_OWNER=<owner> GH_TEST_REPO=<repo> cargo test --test repositories_live --test pull_requests_live -- --nocapture`

## 結果
- unit/integration/live test すべて成功。
- `gh` 実 API 経由で `repo list` / `pr list` が正常に実行されることを確認。

## 次アクション
1. `features/issues` を同じ構造で実装。
2. `pr` の destructive 相当操作ポリシー（必要なら merge 追加ガード）を検討。
3. 契約テストを `gh` バージョン差分検知に使える形に拡張。
