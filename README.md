# gh-client

`gh` (GitHub CLI) をバックエンドに使い、GitHub Web UI で提供される「自分のリポジトリ管理機能」を GUI から操作するためのアプリケーション設計ドキュメントです。

## 目的
- GitHub 操作を GUI で統合し、CLI に不慣れでも管理操作を完結できるようにする
- `gh` と `gh api` を第一選択にし、必要に応じて `git` コマンドを併用する
- GitHub UI の主要機能を、自分が管理するリポジトリ向けに段階的に網羅する

## ドキュメント
- [機能リスト / GitHub UI 対応表](docs/features.md)
- [システム設計](docs/architecture.md)
- [バックエンド設計 (`gh` 実行基盤)](docs/backend.md)
- [実装計画（マイルストーン）](docs/implementation-plan.md)
- [Memory Log](docs/memory/README.md)
- [TODO](TODO.md)
- [ROADMAP](ROADMAP.md)
- [AGENT 運用ガイド](AGENT.md)

## 開発方針（要約）
- まずは PR / Issue / Actions / Releases / Repository Settings を優先実装
- すべての操作は「安全なコマンド実行ラッパー」経由で行う
- API 結果は正規化し、UI はバックエンド API のみを参照する
- 最優先は「不具合の検知容易性」と「セキュリティ安全性」とする
- バックエンドを feature-based に先行完成し、フロントエンドは後付け実装する

## 実装状況（バックエンド）
- `repositories`: list/create/delete
- `pull_requests`: list/create/review/merge
- `issues`: list/create/comment/close/reopen
- `actions`: workflow list/run list/rerun/cancel
- `releases`: list/create/delete
- `settings`: collaborators list/add/remove
