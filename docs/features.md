# 機能リスト（Cross-Repo Review Console）

## スコープ定義
- 対象: 複数リポジトリにまたがる Issue / Pull Request の検索・一覧・レビュー操作
- 対象ユーザー: `admin` / `maintain` / `write` 権限で日常的に triage/review する開発者
- 非対象: GitHub 全領域の網羅（Actions/Releases/Settings 等を主導線にしない）

## 優先度
- P0: MVP で必須
- P1: MVP 直後の高速化・運用性強化
- P2: チーム運用拡張

## コア機能マトリクス
| 領域 | UI 機能 | 優先度 | 実現方式 | 備考 |
|---|---|---:|---|---|
| Cross-Repo Inbox | Issue / PR 横断一覧 | P0 | `gh issue list` / `gh pr list` + `gh api` | 複数 repo を統合して同一リスト表示 |
| Cross-Repo Inbox | 横断検索・フィルタ | P0 | backend query 正規化 | owner/repo, state, label, assignee, reviewer, updated_at |
| Cross-Repo Inbox | 保存ビュー | P0 | ローカル永続化 + 条件テンプレート | 例: 「自分宛レビュー待ち」 |
| Triage | Issue quick action | P0 | `gh issue close/reopen/edit/comment` | 一覧から直接操作 |
| Review | PR quick action | P0 | `gh pr review/close/reopen/merge` | approve/request changes/comment |
| Review | PR 詳細 + 会話 | P0 | `gh pr view` + `gh api` | issue/review comment を統合表示 |
| Diff | ファイル一覧 + unified diff | P0 | `gh api` + `gh pr diff` | `patch` 欠損時は raw diff fallback |
| Workflow | キーボードショートカット | P1 | frontend shortcut layer | 主要操作を 1 キー実行 |
| Workflow | 一括操作（安全ガード付き） | P1 | backend batch command | close/label/assignee など |
| Queue | 優先度表示（滞留ハイライト） | P1 | ルール評価 + 表示属性 | SLA 風ルール（24h 超過など） |
| Audit | 操作履歴トラッキング | P1 | 既存 audit log | だれが何を実行したか追跡 |
| Team Ops | チーム別ビュー | P2 | saved view 拡張 | レビュアーグループ単位の可視化 |
| Team Ops | 通知連携 | P2 | webhook/外部連携 | Slack など |

## 画面構成（P0）
1. Inbox（3レーン）
- 左: 検索条件 / 保存ビュー
- 中央: Issue/PR 横断一覧
- 右: 詳細・会話・差分・操作

2. Diff Viewer
- ファイルツリー
- unified diff
- コメント位置ジャンプ

3. Activity Log
- 操作結果
- 失敗理由の正規化表示

## 非機能要件
- 速度: 初回一覧表示から最初の操作まで 30 秒未満を目標
- 安全性: 破壊系操作は確認フロー + 監査ログ必須
- 冪等性: 更新系 API は `request_id` で二重実行防止
- 安定性: API 失敗時に指数バックオフ + 再試行
- 可観測性: command 単位の成功率 / レイテンシを追跡

## 非対象（明示）
- Actions, Releases, Settings を中心とした管理コンソール化
- 組織課金・監査・SSO 等の org 全体管理
- コード編集 IDE 機能

## リリースフェーズ
1. Phase F1（MVP）
- 横断一覧 / 横断検索 / 保存ビュー
- Issue close/reopen/comment
- PR approve/request changes/comment/close
- 基本 diff 閲覧

2. Phase F2（速度改善）
- ショートカット強化
- 一括操作
- 滞留ハイライト

3. Phase F3（運用拡張）
- チームビュー
- 通知連携
