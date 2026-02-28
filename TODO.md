# TODO

## Now
- [x] Backend workspace を初期化する。
- [x] `core`（registry/executor/policy/observability/error）を実装する。
- [x] `features/repositories` を実装する。
- [x] `repositories` の unit/integration/live test を通す。
- [x] `docs/memory/` の運用を開始する。

## Next
- [ ] `features/pull_requests` を実装する。
- [ ] `features/issues` を実装する。
- [ ] `features/actions` を実装する。
- [ ] `features/releases` を実装する。
- [ ] `features/settings` を実装する。

## Test/Quality
- [ ] destructive command 一覧を明示し、`SAFE_TEST_MODE` ルールをテストに固定する。
- [ ] `gh --json` の契約テストを feature ごとに増やす。
- [ ] CI に security check（`cargo audit`, `cargo deny`）を組み込む。
