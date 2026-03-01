# 2026-03-01 Backend Hardening and E2E

## 何を変更したか
- `settings` feature を全面拡張。
  - 追加: `secrets`（list/set/delete）
  - 追加: `variables`（list/set/delete）
  - 追加: `webhooks`（list/create/ping/delete）
  - 追加: `branch protection`（get/update）
  - 追加: `deploy keys`（list/add/delete）
  - 追加: `dependabot alerts`（list）
- `settings` の `command handler` を上記機能に合わせて拡張。
- `CommandRegistry` の default command 定義を実装に合わせ、期待件数テストを `54` に更新。
- 既存拡張済み feature（repositories/pull_requests/issues/actions/releases）と command id の対応を確認し、未使用 command id を解消。
- cross-feature の live E2E を追加（`src-tauri/tests/e2e_live.rs`）。
- `settings_live` を拡張し、read 系の実操作確認対象を拡大。
- `AGENT.md` / `TODO.md` / `ROADMAP.md` を最新状態へ更新。

## なぜ変更したか
- ユーザー方針（Backend First / feature-based / 本番相当テスト優先）に従い、管理系機能の空白を埋めるため。
- `gh` backend として機能不足だった settings 領域を実装し、GUI 側が後付け可能な状態にするため。
- 断片的 live test だけでなく、複数 feature を横断する read-only 実行経路を定期的に検証可能にするため。

## 何を検証したか
- `cargo fmt --all`
- `cargo test`（unit + integration + live tests with skip logic）
- 実操作 live test（実行済み）:
  - `OWNER=$(gh api user --jq .login)`
  - `REPO=$(gh repo list "$OWNER" --json name --limit 1 --jq '.[0].name')`
  - `GH_CLIENT_LIVE_TEST=1 GH_TEST_OWNER="$OWNER" GH_TEST_REPO="$REPO" cargo test --test repositories_live --test pull_requests_live --test issues_live --test actions_live --test releases_live --test settings_live --test e2e_live -- --nocapture`
- 実行時の対象値:
  - `OWNER=ymuichiro`
  - `REPO=koto-type`
- 上記 live テストはすべて成功。

## 発見した課題
- branch protection 更新 API は `PUT` 仕様上、部分更新時にも既存状態を考慮する必要があるため、get->merge->put の流れにした。さらに厳密には status checks/restrictions を扱う UI 契約が必要。
- CI へ security checks（`cargo audit`, `cargo deny`）は未統合。

## 次のアクション
1. security CI（`cargo audit`, `cargo deny`）を導入。
2. レート制限/ネットワーク障害時の再試行・分類テストを増強。
3. backend command payload 契約を固定し、frontend attach を開始。
