# 実装計画

## 方針（確定）
- Backend First: バックエンド完成までフロントエンド実装は最小限に留める
- Feature-Based: 機能ごとにモジュールを完結実装し、十分な単体テスト通過後に統合
- Test Realism First: テストは本番相当の実操作を基本とする
- Safety Exception: 削除系など破壊的操作はテスト時のみダミー化する

## 現在ステータス（2026-03-01）
- Phase A: 完了
- Phase B: 完了（P0/P1 + P2 の backend module 実装済み）
- Phase C: 完了（cross-feature E2E / resilience / security CI）
- Phase D: 未着手（次フェーズ）

## 開発フェーズ
1. Phase A: Core Foundation（最優先）
- `CommandRegistry`, `Executor`, `PolicyGuard`, `ErrorModel`, `Observability` を実装
- `trace_id`/`request_id`、監査ログ、`SAFE_TEST_MODE` を先に実装

完了条件:
- `gh` 実行が whitelist のみで動作
- 共通エラー分類が全 command で統一
- 構造化ログで失敗原因を追跡可能

2. Phase B: Feature Modules（バックエンド本体）
- 実装順: `repositories` -> `pull_requests` -> `issues` -> `actions` -> `releases` -> `settings`
- 各モジュールは単独で開発・検証し、完了後に統合

各モジュールの完了条件:
- 単体テスト: 正常系/異常系/境界値/権限不足を網羅
- 契約テスト: `gh --json`/`gh api` の期待フィールドが固定化されている
- 実操作テスト: 専用 GitHub テスト repo で主要ユースケースが成功
- 監査要件: 重要操作が audit log に正しく記録される

3. Phase C: Backend Integration Hardening（完了）
- 全モジュール統合後に cross-feature シナリオを検証
- レート制限、ネットワーク障害、`gh` バージョン差分の耐性確認

完了条件:
- E2E 主要導線（PR 作成->レビュー->マージ、Issue 運用、Actions rerun、Release 作成）が成功
- Security CI (`cargo audit`, `cargo deny`) と権限回避テストが常時グリーン

4. Phase D: Frontend Implementation（後付け / 次フェーズ）
- バックエンド API/command 契約に従って UI を実装
- UI 側ロジックは薄く保ち、業務ロジックは追加しない

完了条件:
- 既存バックエンドテストを壊さずに UI から全主要機能を操作可能

## 実装単位（feature-based）
- `src-tauri/src/features/{feature}/command.rs`
- `src-tauri/src/features/{feature}/service.rs`
- `src-tauri/src/features/{feature}/dto.rs`
- `src-tauri/src/features/{feature}/tests/*`

## テスト実行ルール
1. 実操作テスト（デフォルト）
- テスト専用リポジトリに対して create/update/list/view/comment/review/merge/rerun/cancel を実行

2. 破壊操作テスト（例外）
- delete 系は `SAFE_TEST_MODE=true` で no-op adapter に差し替え
- 戻り値、権限判定、監査ログ、事前確認フローのみ検証

3. CI ゲート
- PR マージ条件: unit + contract + integration + security すべて成功
- 任意の feature module でテスト不通過なら統合禁止

## 品質ゲート
- Unit: feature module ごとに十分なケース網羅（正常/異常/境界/権限）
- Integration: `gh` 変換ロジックの golden test 固定
- E2E: 本番相当テスト repo で主要導線を通す
- Security: 依存脆弱性検査 + 権限回避テスト + 監査ログ検証

## リスクと対策
- `gh` 出力差分: フィールド固定の契約テストを必須化
- 破壊操作の誤実行: `SAFE_TEST_MODE` と command_id ベース no-op で遮断
- モジュール間の結合バグ: feature 完了後に段階的統合し、cross-feature E2E で検出
