# 実装計画

## 方針（更新: 2026-03-08）
- Product Focus First: 複数 repo の Issue/PR 横断処理を最優先する
- Reuse Existing Assets: 既存 backend 機能は維持し、UI/UX 投資を集中する
- Backend Reliability First: `gh` 実行の安全性・エラー正規化・監査ログを継続強化する
- Thin UI: UI は高速操作導線に専念し、業務ロジックは backend に寄せる
- Test Realism First: 可能な限り live 条件で主要導線を検証する

## 現在ステータス（2026-03-08）
- Foundation〜Frontend Attach（旧 Phase A-D）は完了
- broad feature backend（repositories/pull_requests/issues/actions/releases/settings + P2）は実装済み
- 次の実装対象は Cross-Repo Review Console の UX と運用速度

## 開発フェーズ（今後）
1. Phase E: Cross-Repo Inbox MVP
- Issue/PR 横断一覧 API を主導線に再編
- owner/repo, state, label, assignee, reviewer, updated_at の検索条件を統一
- 保存ビュー（レビュー待ち/滞留/自分担当）を導入

完了条件:
- 1画面で複数 repo の Issue/PR を一覧できる
- 保存ビューから 1 操作で一覧再現できる
- 一覧初動から最初の操作まで 30 秒未満（目標）

2. Phase F: Quick Action & Review Flow
- 一覧から Issue/PR への quick action を追加
- PR review（approve/request changes/comment）を右ペインで完結
- diff への遷移を最短化（一覧 -> 詳細 -> diff）

完了条件:
- close/approve/review の平均操作ステップを既存 UI より削減
- 主要操作がショートカット経由で実行可能
- 監査ログに操作結果が正しく記録される

3. Phase G: Queue Intelligence
- 滞留検知（例: 24h 超過）と優先度表示
- 一括処理（安全ガード付き）を導入
- 活動履歴ビューを強化

完了条件:
- 期限超過アイテムを自動で可視化
- 一括操作の失敗時に対象ごとの結果が追跡可能
- 取りこぼし率を定点観測できる

## 実装単位（feature-based）
- `src-tauri/src/features/pull_requests/*`
- `src-tauri/src/features/issues/*`
- `src-tauri/src/features/repositories/*`（横断一覧の補助情報）
- `ui/src/pages/*`
- `ui/src/components/*`
- `ui/src/core/*`

## テスト実行ルール
1. 実操作テスト（デフォルト）
- 横断一覧、Issue/PR 更新系、レビュー系を live 条件で優先検証

2. 破壊操作テスト（例外）
- delete 系は `SAFE_TEST_MODE=true` で no-op に差し替え
- 事前確認と監査ログの整合性を重点検証

3. CI ゲート
- PR マージ条件: unit + integration + e2e + security を維持
- 横断一覧と quick action の回帰テストは必須

## 品質ゲート
- Unit: 条件変換・DTO 正規化・失敗分類を検証
- Integration: `gh` 出力の契約固定（`--json` / `gh api`）
- E2E: 主要導線（検索 -> 一覧 -> 詳細 -> 操作 -> 反映）を通す
- Security: 依存脆弱性検査 + 権限回避テスト + 監査ログ検証

## リスクと対策
- `gh` 出力差分: 契約テストを維持し、変換層で吸収
- 操作速度と安全性の衝突: quick action は確認ポリシーをレベル別に設計
- 横断一覧の API 負荷: 短期キャッシュ + 再試行 + バックオフで緩和
