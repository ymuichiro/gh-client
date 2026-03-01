# 2026-03-01 PR Chat/Diff Expansion

## 目的
- `pull_requests` backend を GitHub Web UI 相当の会話・diff 閲覧に近づける。
- 既存の `list/create/edit/review/merge/close/reopen` に加えて、chat 形式の返信運用と thread 状態管理を実装する。
- payload contract を v2 に更新し、フロント接続時の command surface を固定化する。

## 変更内容
- `src-tauri/src/features/pull_requests` を拡張:
  - `dto.rs`
    - 追加 DTO: `PullRequestDetail`, `PullRequestComment`, `PullRequestReviewThread`, `PullRequestDiffFile`, `PullRequestRawDiff`
    - 追加 parser: detail / issue comments / review comments / review threads / diff files / raw diff
  - `service.rs`
    - 追加 API: `view`, `list_issue_comments`, `create_issue_comment`, `list_review_comments`, `create_review_comment`, `reply_review_comment`, `list_review_threads`, `resolve_review_thread`, `unresolve_review_thread`, `list_diff_files`, `get_raw_diff`
    - 追加 input validation: `CommentPullRequestInput`, `CreateReviewCommentInput`, `ReplyReviewCommentInput`, `ResolveReviewThreadInput`
  - `command.rs`
    - 新規 service API に対応する command handler メソッドを追加
- `src-tauri/src/core/command_registry.rs`
  - PR 拡張用 command を 11 個追加:
    - `pr.view`
    - `pr.comments.list`, `pr.comments.create`
    - `pr.review_comments.list`, `pr.review_comments.create`, `pr.review_comments.reply`
    - `pr.review_threads.list`, `pr.review_threads.resolve`, `pr.review_threads.unresolve`
    - `pr.diff.files.list`, `pr.diff.raw.get`
  - default command 数テストを `76 -> 87` に更新
- `src-tauri/src/contract/mod.rs`
  - `PAYLOAD_CONTRACT_VERSION` を `2026-03-01.v2` へ更新
  - `STABLE_COMMAND_IDS` に新規 PR command を追加
- live/e2e テスト更新:
  - `src-tauri/tests/pull_requests_live.rs`
    - read: list/view/comments/review_threads/diff files/raw diff
    - write (opt-in): issue comment create, review comment reply, thread resolve/unresolve
    - write 実行は `GH_CLIENT_LIVE_WRITE_TEST=1` 前提
  - `src-tauri/tests/e2e_live.rs`
    - 既存 read-only flow に PR detail/comments/threads/diff を追加
- ドキュメント更新:
  - `docs/features.md`, `docs/backend.md`, `docs/payload-contract.md`, `README.md`, `TODO.md`, `ROADMAP.md`

## 設計意図
- 認証は引き続き `gh auth login` セッション依存とし、アプリ側で token を保持しない。
- UI が扱いやすいよう、コメントは `kind=issue_comment|review_comment` の正規化 DTO で返却。
- diff は構造化（files）と raw（unified）を両方提供し、`patch` 欠損時の fallback を確保。
- review thread 操作は GraphQL (`resolveReviewThread` / `unresolveReviewThread`) で実装。

## 実施した検証
- `cargo fmt --all`
- `cargo test`
  - unit/contract/integration/live test スイートまで成功（環境変数未設定時は live が skip 成功）

## 発見した課題
- review thread の live write テストは、対象 thread/comment id がない場合に skip になる。
- UI から使いやすくするため、将来はページング情報や時系列統合コメント API（issue+review の単一 stream）があるとよい。

## 次のアクション
1. フロントエンド実装時に payload contract v2 を採用し、PR 会話 UI を接続する。
2. review comments create の live write 検証を安定化するため、専用 fixture PR 運用を検討する。
3. PR 画面の差分描画で `patch=None` ケースの raw diff fallback を明示実装する。
