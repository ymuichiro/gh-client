# 2026-02-28 Backend Bootstrap

## 目的
- Backend First 方針に基づき、feature-based 実装の最初の土台を作る。
- テストしやすさと安全性を優先して `core` + `repositories` を先行実装する。

## 変更内容
- Rust workspace を初期化し、`src-tauri` をライブラリ crate として作成。
- `core` を実装:
  - `command_registry`
  - `executor`
  - `policy_guard`
  - `observability`
  - `error`
- `features/repositories` を実装:
  - `dto`（JSON parse）
  - `service`（list/create/delete）
  - `command`（handler）
- live test を追加:
  - `tests/repositories_live.rs`

## 設計意図
- `CommandRegistry` で実行コマンドを whitelist 化し、任意コマンド実行を防ぐ。
- `SAFE_TEST_MODE` で destructive command を no-op にし、破壊系テスト事故を防ぐ。
- `PolicyGuard` で `viewer/write/admin` 権限を明示的にチェックする。
- `AppError` を統一して失敗分類と追跡性を上げる。

## 実施した検証
- `cargo fmt --all`
- `cargo test`（unit + integration）
- `GH_CLIENT_LIVE_TEST=1` で real `gh repo list` 実操作テスト

## 発見事項
- `gh repo list` は `--owner` フラグを受け付けない。
- owner は位置引数で渡す必要がある。
- これを修正して live test 成功を確認。

## 現在の到達点
- `core` + `repositories` は実装済み、テスト通過済み。
- 次フェーズは `pull_requests` feature 実装。

## 運用整備（同日追記）
- `AGENT.md` を追加し、継続開発の前提知識と必須ルールを固定化。
- `docs/memory/README.md` を追加し、履歴記録方式を明文化。
- `TODO.md` と `ROADMAP.md` を追加し、短期作業と中期計画を保存。

## 次アクション
1. `features/pull_requests` を同じ構造で実装。
2. `SAFE_TEST_MODE` の対象 destructive command を feature ごとに増やす。
3. 契約テスト（`gh --json` フィールド固定）を feature 単位で追加する。
