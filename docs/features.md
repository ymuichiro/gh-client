# 機能リスト（GitHub UI 自身のリポジトリ操作）

## スコープ定義
- 対象: 自分が `admin` / `maintain` / `write` 権限を持つリポジトリ
- 非対象: 組織全体の課金・監査・SSO 管理など、リポジトリ境界を超える管理機能

## 優先度
- P0: 最初のリリースで必須
- P1: 早期拡張
- P2: 網羅性向上（後続）

## GitHub UI 対応マトリクス
| 領域 | UI 機能 | 優先度 | 実現方式 | 備考 |
|---|---|---:|---|---|
| リポジトリ一覧 | 自分のリポジトリ一覧/検索/フィルタ | P0 | `gh repo list --json` | owner, visibility, language 等で絞り込み |
| リポジトリ作成 | 新規 repo 作成（public/private/template） | P0 | `gh repo create` | 初期化オプション対応 |
| リポジトリ基本情報 | description/homepage/topics 編集 | P0 | `gh repo edit` + `gh api` | Topics は API 併用 |
| Code | ブランチ一覧/作成/削除 | P0 | `gh api` + `git` | 既定ブランチ変更を含む |
| Code | コミット履歴/差分閲覧 | P0 | `gh api` + `git log/show` | ローカル clone がある場合高速化 |
| Pull Requests | 一覧/検索/詳細表示 | P0 | `gh pr list/view --json` | レビュー状態フィルタ |
| Pull Requests | 作成（draft含む）/編集/クローズ | P0 | `gh pr create/edit/close` | テンプレート読み込み対応 |
| Pull Requests | レビュー（approve/request changes/comment） | P0 | `gh pr review` | |
| Pull Requests | マージ（merge/squash/rebase） | P0 | `gh pr merge` | ブランチ削除オプション |
| Issues | 一覧/検索/詳細表示 | P0 | `gh issue list/view --json` | assignee, labels, milestone |
| Issues | 作成/編集/コメント/close/reopen | P0 | `gh issue create/edit/comment/reopen/close` | |
| Actions | Workflow 一覧/Run 一覧/ログ閲覧 | P0 | `gh workflow list` + `gh run list/view` | ログダウンロード |
| Actions | Run 再実行/キャンセル | P0 | `gh run rerun/cancel` | |
| Releases | Release 一覧/作成/編集/削除 | P0 | `gh release list/create/edit/delete` | Asset upload/delete 含む |
| Settings | Collaborators 管理 | P1 | `gh api` | 招待/権限変更/削除 |
| Settings | Secrets / Variables（Actions） | P1 | `gh secret` + `gh variable` | repo/environment 対応 |
| Settings | Webhooks 管理 | P1 | `gh api` | create/update/ping/delete |
| Settings | Branch Protection | P1 | `gh api` | rule の作成/更新 |
| Settings | Deploy Keys | P1 | `gh api` | read-only/write 指定 |
| Security | Dependabot alerts 一覧/対応導線 | P1 | `gh api` | dismiss/reopen は API |
| Projects | リポジトリ紐づけ Project 操作 | P2 | `gh api graphql` | backend 実装済（read:project scope が必要） |
| Discussions | 一覧/作成/回答/クローズ | P2 | `gh api graphql` | backend 実装済 |
| Wiki | 有効化状態確認と導線 | P2 | `gh api` + 外部エディタ | backend 実装済（編集導線は後続） |
| Insights | Traffic（views/clones）表示 | P2 | `gh api` | backend 実装済（read-only） |
| Pages | Pages 設定（source/build） | P2 | `gh api` | backend 実装済 |
| Rulesets | ルールセット管理 | P2 | `gh api` | backend 実装済 |

## 画面単位の機能一覧（P0）
1. Dashboard
- 自分の repo 一覧
- 最近の PR / Issue / Actions 実行状況

2. Repository Home
- 基本情報編集（description, topics, homepage, visibility）
- README / デフォルトブランチ情報表示

3. Pull Request
- 一覧・詳細・レビュー・マージ
- 差分表示、チェック結果表示

4. Issues
- 一覧・詳細・コメント
- ラベル/マイルストーン/アサイン操作

5. Actions
- workflow 一覧
- run 詳細（jobs/logs）
- rerun/cancel

6. Releases
- tag/release 管理
- assets 管理

## 非機能要件（機能実装に直結）
- 操作ログ: 誰が何を実行したかをローカルに監査ログ化
- 冪等性: 連打時の二重実行を防止（client request id）
- レート制限対策: ETag/短期キャッシュ + 失敗時の指数バックオフ
- 権限制御: 実行前に repo 権限 (`viewer/write/admin`) を確認

## リリースフェーズ
1. Phase 1（P0 Core）
- Repo, PR, Issue, Actions, Release の CRUD と主要操作

2. Phase 2（P1 Admin）
- Collaborators, Secrets/Variables, Webhooks, Branch Protection

3. Phase 3（P2 Coverage）
- Discussions, Projects, Insights, Pages, Rulesets（backend 実装済）
