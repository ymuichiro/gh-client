# ROADMAP

## Phase 0: Foundation（完了）
- Rust workspace 構築
- core modules 実装
- repositories feature 実装
- 実操作テストの土台作成

## Phase 1: Collaboration Core（進行中）
- pull_requests feature（完了）
- issues feature
- 目標: 日常開発で必要な操作を backend で完結

## Phase 2: Delivery Core
- actions feature
- releases feature
- 目標: CI/CD と配布操作を backend で完結

## Phase 3: Admin Core
- settings feature（collaborators/secrets/webhooks/protection など）
- 目標: 管理系機能を安全な権限制御付きで提供

## Phase 4: Integration Hardening
- cross-feature E2E
- レート制限・ネットワーク障害・`gh` 差分への耐性確認
- セキュリティ検査を CI で常時グリーン化

## Phase 5: Frontend Attach
- backend 契約に追従する UI 実装
- UI は薄い層に限定し、業務ロジックを持たせない
