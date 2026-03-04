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

## 実装状況（バックエンド + フロントエンド）
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

フロントエンド:
- `Tauri + React + TypeScript` 構成
- 主要機能は feature ページ、長尾は `Command Console` で全 `STABLE_COMMAND_IDS` に到達可能
- UI 言語: 日本語/英語の切替
- 破壊系操作: 2段階確認モーダル
- 実行履歴: ローカル監査ログ（request_id / command_id / status）

## ローカル起動
1. 依存関係
- Rust（stable）
- Node.js / npm
- GitHub CLI `gh`
- `gh auth status` が成功すること

2. 依存インストール
```bash
npm --prefix ui install
```

3. Web UI（mock 実行）起動
```bash
VITE_EXECUTION_MODE=mock npm --prefix ui run dev -- --host 127.0.0.1 --port 5173
```

4. デスクトップ（Tauri）起動
```bash
cargo run --manifest-path src-tauri/Cargo.toml --features desktop --bin gh-client-desktop
```

5. 参考: Tauri CLI を使う場合
```bash
cargo tauri dev --features desktop
```

## テスト方法
### 1. Rust テスト（unit + live）
```bash
cargo test
```

### 2. フロント unit/integration（Vitest）
```bash
npm --prefix ui run test
```

### 3. フロント E2E（Playwright / mock）
初回のみ browser binary をインストール:
```bash
npx --prefix ui playwright install chromium
```

その後に実行:
```bash
npm --prefix ui run e2e
```

### 4. フロント E2E（Playwright / live read）
```bash
OWNER=$(gh api user --jq .login)
REPO=$(gh repo list "$OWNER" --json name --limit 1 --jq '.[0].name')

GH_CLIENT_LIVE_TEST=1 \
GH_TEST_OWNER="$OWNER" \
GH_TEST_REPO="$REPO" \
npm --prefix ui run e2e:live
```

### 5. フロント E2E（Playwright / live write opt-in）

```bash
OWNER=$(gh api user --jq .login)
REPO=$(gh repo list "$OWNER" --json name --limit 1 --jq '.[0].name')

GH_CLIENT_LIVE_TEST=1 \
GH_CLIENT_LIVE_WRITE_TEST=1 \
GH_TEST_OWNER="$OWNER" \
GH_TEST_REPO="$REPO" \
npm --prefix ui run e2e:write
```

## 補足（安全性）
- 破壊系コマンドは `SAFE_TEST_MODE=true` のとき no-op になります（テスト時の安全措置）。
- 認証は `gh` に依存し、このアプリは GitHub token を保存・管理しません。
