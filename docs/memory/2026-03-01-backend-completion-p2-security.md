# 2026-03-01 Backend Completion (Contract/Security/Resilience/P2)

## 何を変更したか
- 未完了項目だった4点をバックエンドで実装完了した。

### 1) フロント接続用 payload 契約固定
- `src-tauri/src/contract/mod.rs` を追加。
  - `PAYLOAD_CONTRACT_VERSION = 2026-03-01.v1`
  - `FrontendCommandEnvelope`
  - `STABLE_COMMAND_IDS`
  - `validate_registry_contract()`
- 契約ドキュメントを追加: `docs/payload-contract.md`
- 契約と registry の一致をテストで固定。

### 2) security CI (`cargo audit`, `cargo deny`) 組み込み
- `.github/workflows/security.yml` を追加。
- `.github/workflows/ci.yml` を追加（fmt/test）。
- `deny.toml` を追加。
- crate license を `AGPL-3.0-only` で明示（`src-tauri/Cargo.toml`）。

### 3) レート制限/ネットワーク障害耐性強化
- `src-tauri/src/core/executor.rs` を強化。
  - 失敗分類: `AuthRequired / PermissionDenied / NotFound / RateLimited / NetworkError / UpstreamError / ExecutionError`
  - retryable エラー時の retry/backoff 実装
  - scope不足（required scopes）を権限エラーとして分類
- `ErrorCode::NetworkError` 追加（`src-tauri/src/core/error.rs`）。
- executor テストを拡充（分類/再試行/非再試行）。

### 4) P2 coverage 実装
- 新規 feature module を追加:
  - `projects`（list/items/add）
  - `discussions`（categories/list/create/close/answer）
  - `wiki`（get/update）
  - `pages`（get/create/update/delete）
  - `rulesets`（list/get/create/update/delete）
  - `insights`（traffic views/clones）
- `CommandRegistry` に P2 command を追加。
- `features/mod.rs` を更新。
- P2 live test を追加: `src-tauri/tests/p2_live.rs`
  - token scope不足や未有効機能は skip で扱う。

## なぜ変更したか
- ユーザー指定の未完了4項目（契約固定、security CI、耐障害性、P2網羅）をバックエンド完了条件として満たすため。

## 何を検証したか
- `cargo fmt --all`
- `cargo test`（unit + existing live tests + p2_live）
- 実操作テスト:
  - `GH_CLIENT_LIVE_TEST=1 GH_TEST_OWNER=... GH_TEST_REPO=... cargo test --test repositories_live --test pull_requests_live --test issues_live --test actions_live --test releases_live --test settings_live --test e2e_live --test p2_live -- --nocapture`
- security checks:
  - `cargo audit`
  - `cargo deny check advisories bans licenses sources`

## 発見した課題
- Projects GraphQL (`projectsV2`) は token に `read:project` scope が必要。
  - この scope がない場合、`p2_live` は skip として扱う。
- Pages 未設定 repo では `pages.get` が `404` を返す。
  - `p2_live` は skip として扱う。

## 次のアクション
1. フロントエンド（Tauri）起動構成の追加。
2. payload contract v1 を使った UI ↔ backend 接続層を実装。
3. GUI 操作 E2E（Playwright）追加。
