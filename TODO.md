# TODO

## Now
- [x] Backend workspace を初期化する。
- [x] `core`（registry/executor/policy/observability/error）を実装する。
- [x] `features/repositories` を実装する。
- [x] `repositories` の unit/integration/live test を通す。
- [x] `docs/memory/` の運用を開始する。

## Next
- [x] `features/pull_requests` を実装する。
- [x] `features/issues` を実装する。
- [x] `features/actions` を実装する。
- [x] `features/releases` を実装する。
- [x] `features/settings` を実装する。
- [ ] cross-feature E2E シナリオを実装する。
- [ ] フロントエンド接続用の command payload 契約を固定化する。

## Test/Quality
- [ ] destructive command 一覧を明示し、`SAFE_TEST_MODE` ルールをテストに固定する。
- [ ] `gh --json` の契約テストを feature ごとに増やす。
- [ ] CI に security check（`cargo audit`, `cargo deny`）を組み込む。
