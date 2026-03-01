# TODO

## Now
- [x] Backend workspace を初期化する。
- [x] `core`（registry/executor/policy/observability/error）を実装する。
- [x] `features/repositories` を実装する。
- [x] `repositories` の unit/integration/live test を通す。
- [x] `docs/memory/` の運用を開始する。

## Next
- [x] `features/pull_requests` を実装する。
- [x] `pull_requests` の chat/diff 拡張（comments/review_threads/diff files/raw diff）を実装する。
- [x] `features/issues` を実装する。
- [x] `features/actions` を実装する。
- [x] `features/releases` を実装する。
- [x] `features/settings` を実装する。
- [x] cross-feature E2E シナリオを実装する。
- [x] フロントエンド接続用の command payload 契約を固定化する。
- [x] P2 coverage（Projects/Discussions/Wiki/Pages/Rulesets/Insights）を実装する。

## Test/Quality
- [x] destructive command 一覧を明示し、`SAFE_TEST_MODE` ルールをテストに固定する。
- [x] `gh --json` の契約テストを feature ごとに増やす。
- [x] CI に security check（`cargo audit`, `cargo deny`）を組み込む。
- [x] レート制限/ネットワーク障害時の再試行・エラー分類を executor に実装する。

## Upcoming (Frontend)
- [x] Tauri アプリの最小起動構成（`cargo tauri dev`）を追加する。
- [x] payload contract v2 に沿った UI <-> backend 接続層を実装する。
- [x] 全 `STABLE_COMMAND_IDS` を GUI 到達可能（feature page + command console）にする。
- [x] Playwright E2E（mock read / live read / live write opt-in）を追加する。
