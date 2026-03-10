# gh-client

`gh` (GitHub CLI) をバックエンドに使い、複数リポジトリの Issue / Pull Request 運用を高速化するデスクトップクライアントです。

## 目的
- 複数リポジトリに散らばる Issue / PR を 1 つの作業面で横断管理する
- close / approve / review などの日次運用を最短導線で実行できるようにする
- 差分比較とレビュー文脈を保ったまま判断と操作を完結させる
- 認証情報はアプリに保持せず、`gh auth login` セッションに依存する

## 現在のプロダクトフォーカス
- `Cross-Repo Review Console` として、Issue/PR の横断検索・一覧・操作に集中する
- GitHub の全機能網羅は目標にしない（必要最小限を優先）

## ドキュメント
- [アプリケーションコンセプト（絞り込み版）](docs/application-concept-focused.md)
- [機能リスト（フォーカス後）](docs/features.md)
- [システム設計](docs/architecture.md)
- [バックエンド設計 (`gh` 実行基盤)](docs/backend.md)
- [実装計画（マイルストーン）](docs/implementation-plan.md)
- [ROADMAP](ROADMAP.md)
- [Frontend Payload Contract](docs/payload-contract.md)
- [Memory Log](docs/memory/README.md)
- [TODO](TODO.md)
- [AGENT 運用ガイド](AGENT.md)

## 実装状況（資産）
- backend は `repositories / pull_requests / issues / actions / releases / settings` を含む feature-based 実装済み
- frontend は `Tauri + React + TypeScript` で command 契約に接続済み
- `pull_requests` は review thread / diff files / raw diff まで backend 実装済み
- Playwright E2E（mock/live）と Rust 側テストを整備済み

注: 実装資産としては広範囲機能を保持しているが、今後の UI/UX と運用導線は Issue/PR 横断処理を優先する。

## 開発方針（要約）
- Product Focus First: 横断 Issue/PR 処理に寄与する改善を最優先
- Safety First: 破壊操作は確認フロー + 監査ログで保護
- Backend Reliability First: `gh` 実行の安全ラッパーとエラー正規化を維持
- Thin UI: 業務ロジックは backend 側に寄せ、UI は操作速度に集中

## ローカル起動

共通コマンドは `Makefile` に集約しています（`make help` で一覧表示）。

よく使う操作:
```bash
make install
make run-desktop
```

`make run-desktop` は UI dev server 起動待ちを内包した 1 コマンド起動です（タイミング依存を排除）。
standalone release 起動（`ui/dist` + `desktop-custom-protocol`）は `make run-desktop-release` を使ってください。
debug 起動を分離したい場合は、別ターミナルで `make dev-web-tauri` を先に起動してから `make run-desktop-debug` を使ってください。

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
make run-desktop
```

5. standalone release 起動（`ui/dist`）
```bash
make run-desktop-release
```

6. debug 起動（`dev-web-tauri` 先行起動が必要）
```bash
make dev-web-tauri
make run-desktop-debug
```

7. 参考: Tauri CLI を使う場合
```bash
cargo tauri dev --features desktop
```

## テスト方法
`Makefile` ショートカット:
```bash
make test-rust
make test-ui
make test-e2e
make test-e2e-live
make test-e2e-write
```

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
