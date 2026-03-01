# ROADMAP

## Phase 0: Foundation（完了）
- Rust workspace 構築
- core modules 実装
- repositories feature 実装
- 実操作テストの土台作成

## Phase 1: Collaboration Core（完了）
- pull_requests feature（完了）
- pull_requests chat/diff 拡張（comments/review_threads/resolve/unresolve/diff files/raw）（完了）
- issues feature（完了）
- 目標: 日常開発で必要な操作を backend で完結

## Phase 2: Delivery Core（完了）
- actions feature（完了）
- releases feature（完了）
- 目標: CI/CD と配布操作を backend で完結

## Phase 3: Admin Core（完了）
- settings feature（collaborators / secrets / variables / webhooks / branch protection / deploy keys / dependabot alerts）
- 目標: 管理系機能を安全な権限制御付きで提供

## Phase 4: Integration Hardening（完了）
- cross-feature E2E（完了）
- レート制限・ネットワーク障害への再試行/分類（完了）
- セキュリティ検査 CI（`cargo audit`, `cargo deny`）常時実行（完了）

## Phase 5: Frontend Attach（完了）
- backend 契約に追従する UI 実装
- UI は薄い層に限定し、業務ロジックを持たせない
- Tauri IPC 入口 `frontend_execute` を追加
- React UI（feature pages + command console）実装
- Playwright E2E（mock/live）実装

## Phase 6: Productization（次フェーズ）
- 配布向け署名/バンドル最適化
- UI/UX の段階的改善（差分閲覧体験、長時間操作の進捗表現）
- live E2E の nightly 自動運用
