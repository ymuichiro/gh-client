# 2026-03-01 Auth Model: GH Session Passthrough

## 何を変更したか
- `features/auth` を追加し、`gh auth status` を参照する API を実装。
  - `src-tauri/src/features/auth/dto.rs`
  - `src-tauri/src/features/auth/service.rs`
  - `src-tauri/src/features/auth/command.rs`
- `CommandRegistry` に `auth.status` を追加。
- frontend payload 契約 (`STABLE_COMMAND_IDS`) に `auth.status` を追加。
- README / backend docs / payload contract docs / AGENT に「アプリは token を保持せず、`gh auth login` セッション依存」であることを明記。

## なぜ変更したか
- 要求仕様として「GitHub token をアプリで持たず、`gh` CUI の薄い wrapper GUI」に統一するため。

## 何を検証したか
- `cargo test` 実行（auth module 追加後に全テスト成功）。
- `features/auth` の unit test で以下を確認。
  - ログイン状態の parse（account / active / scopes）
  - 未ログイン時 (`gh auth login` 案内) の logged_out 変換

## 発見した課題
- `gh auth status` の出力は将来変化し得るため、文字列パースは最小依存にしてある。
- `projectsV2` を使う機能は `read:project` scope が必要。

## 次のアクション
1. フロントエンド起動時に `auth.status` を preflight で呼び、未ログインなら `gh auth login` を案内する。
2. スコープ不足（例: `read:project`）時の GUI ガイド文を整備する。
