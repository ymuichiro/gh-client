# gh-client

`gh` (GitHub CLI) をバックエンドに使い、GitHub Web UI で提供される「自分のリポジトリ管理機能」を GUI から操作するためのアプリケーション設計ドキュメントです。

## 目的
- GitHub 操作を GUI で統合し、CLI に不慣れでも管理操作を完結できるようにする
- `gh` と `gh api` を第一選択にし、必要に応じて `git` コマンドを併用する
- GitHub UI の主要機能を、自分が管理するリポジトリ向けに段階的に網羅する
- アプリ自身は認証情報を保持せず、ローカル `gh auth login` セッションに依存する

## ドキュメント
- [機能リスト / GitHub UI 対応表](docs/features.md)
- [システム設計](docs/architecture.md)
- [バックエンド設計 (`gh` 実行基盤)](docs/backend.md)
- [実装計画（マイルストーン）](docs/implementation-plan.md)
- [Frontend Payload Contract](docs/payload-contract.md)
- [Memory Log](docs/memory/README.md)
- [TODO](TODO.md)
- [ROADMAP](ROADMAP.md)
- [AGENT 運用ガイド](AGENT.md)

## 開発方針（要約）
- まずは PR / Issue / Actions / Releases / Repository Settings を優先実装
- すべての操作は「安全なコマンド実行ラッパー」経由で行う
- API 結果は正規化し、UI はバックエンド API のみを参照する
- 最優先は「不具合の検知容易性」と「セキュリティ安全性」とする
- バックエンドを feature-based に先行完成し、フロントエンドは後付け実装する

## 実装状況（バックエンド）
- `auth`: `gh auth status` の参照（ログイン状態/アカウント/scope を返す）
- `repositories`: list/create/edit/delete, branch list/create/delete, commit list
- `pull_requests`: list/view/create/edit/close/reopen/review/merge, issue/review comment list/create/reply, review thread list/resolve/unresolve, diff files/raw diff
- `issues`: list/create/edit/comment/close/reopen
- `actions`: workflow list/run list/run detail/run logs/rerun/cancel
- `releases`: list/create/edit/delete, asset upload/delete
- `settings`: collaborators, secrets, variables, webhooks, branch protection, deploy keys, dependabot alerts
- `projects` (P2): project list/item list/item add
- `discussions` (P2): category list/discussion list/create/close/answer
- `wiki` (P2): wiki 有効状態の取得/更新
- `pages` (P2): pages 設定の取得/作成/更新/削除
- `rulesets` (P2): ruleset list/get/create/update/delete
- `insights` (P2): traffic views/clones の取得

## ローカルでの起動（現状）
現時点は **バックエンド（Rust crate）中心** で、GUI 本体の起動コマンドはまだありません。  
ローカルでは以下の流れで「ビルドと実動作確認」を行います。

1. 依存関係を準備
- Rust（stable）
- GitHub CLI `gh`
- GitHub 認証済み状態（`gh auth status` が成功すること）

2. ルートでビルド
```bash
cargo build
```

3. バックエンドの動作確認（単体テスト）
```bash
cargo test
```

## テスト方法
### 1. 通常テスト（モック/ユニット中心）
```bash
cargo test
```

### 2. 実 GitHub を使うライブテスト
事前に `gh` 認証を済ませたうえで実行します。

```bash
OWNER=$(gh api user --jq .login)
REPO=$(gh repo list "$OWNER" --json name --limit 1 --jq '.[0].name')

GH_CLIENT_LIVE_TEST=1 \
GH_TEST_OWNER="$OWNER" \
GH_TEST_REPO="$REPO" \
cargo test \
  --test repositories_live \
  --test pull_requests_live \
  --test issues_live \
  --test actions_live \
  --test releases_live \
  --test settings_live \
  --test e2e_live \
  --test p2_live \
  -- --nocapture
```

write 系 live テスト（PR コメント投稿/返信/resolve/unresolve）を含める場合:

```bash
GH_CLIENT_LIVE_TEST=1 \
GH_CLIENT_LIVE_WRITE_TEST=1 \
GH_TEST_OWNER="$OWNER" \
GH_TEST_REPO="$REPO" \
# 返信テストを実行する場合に指定（任意）
GH_TEST_REVIEW_COMMENT_ID="<review_comment_id>" \
# resolve/unresolve の対象を固定したい場合に指定（任意）
GH_TEST_REVIEW_THREAD_ID="<review_thread_id>" \
cargo test --test pull_requests_live -- --nocapture
```

### 3. feature 単位でライブテストを実行
```bash
GH_CLIENT_LIVE_TEST=1 GH_TEST_OWNER="<owner>" GH_TEST_REPO="<repo>" cargo test --test settings_live -- --nocapture
```

## 補足（安全性）
- 破壊系コマンドは `SAFE_TEST_MODE=true` のとき no-op になります（テスト時の安全措置）。
- 認証は `gh` に依存し、このアプリは GitHub token を保存・管理しません。
