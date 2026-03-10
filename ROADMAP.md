# ROADMAP

## フォーカス更新（2026-03-08）
プロダクトの主目的を「GitHub 機能網羅」から「複数リポジトリの Issue/PR 横断処理の高速化」へ再定義。
既存の backend 資産は維持しつつ、今後の UI/UX 投資は Cross-Repo Review Console に集中する。

## 完了済みフェーズ（履歴）

### Phase 0: Foundation（完了）
- Rust workspace 構築
- core modules 実装
- repositories feature 実装
- 実操作テストの土台作成

### Phase 1: Collaboration Core（完了）
- pull_requests feature（完了）
- pull_requests chat/diff 拡張（comments/review_threads/resolve/unresolve/diff files/raw）（完了）
- issues feature（完了）

### Phase 2: Delivery Core（完了）
- actions feature（完了）
- releases feature（完了）

### Phase 3: Admin Core（完了）
- settings feature（collaborators / secrets / variables / webhooks / branch protection / deploy keys / dependabot alerts）

### Phase 4: Integration Hardening（完了）
- cross-feature E2E（完了）
- レート制限・ネットワーク障害への再試行/分類（完了）
- セキュリティ検査 CI（`cargo audit`, `cargo deny`）常時実行（完了）

### Phase 5: Frontend Attach（完了）
- backend 契約に追従する UI 実装
- Tauri IPC 入口 `frontend_execute` を追加
- React UI（feature pages + command console）実装
- Playwright E2E（mock/live）実装

## これからのフェーズ（優先）

### Phase 6: Cross-Repo Inbox MVP（次フェーズ）
- Issue/PR 横断一覧の主画面化
- 保存ビュー（レビュー待ち/滞留/自分担当）
- 一覧からの quick action（close/reopen/approve/request changes/comment）
- 目標: triage 開始までの時間を最短化

### Phase 7: Review Velocity UX
- 3レーン UI（検索条件 / 横断一覧 / 詳細+diff）
- キーボード中心の操作導線
- diff 閲覧導線の短縮（一覧から 1 アクション遷移）
- 目標: 1件あたりのレビュー操作ステップ削減

### Phase 8: Queue Intelligence
- 滞留ハイライト（例: 24h 超過）
- 優先度並び替えと SLA 風ルール
- 監査ログの可視化強化
- 目標: 期限超過 PR/Issue の取りこぼし率を低減

### Phase 9: Team Operations（拡張）
- チーム別ビュー
- 通知連携（Slack など）
- 目標: 個人最適からチーム運用最適へ拡張
