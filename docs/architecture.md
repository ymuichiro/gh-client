# システム設計

## 推奨アーキテクチャ
- フロントエンド: React + TypeScript（デスクトップ UI）
- アプリシェル: Tauri（軽量デスクトップ配布）
- バックエンド: Rust（Tauri core command 層）
- 実行基盤: `gh` / `gh api` / `git` を安全ラッパー経由で実行
- 永続化: SQLite（キャッシュ、ジョブ状態、監査ログ）

## 構成
1. UI Layer
- 画面描画、入力、状態管理
- Tauri command 経由でのみ操作（`gh` を直接叩かない）

2. Tauri Command Layer
- IPC 境界の入力検証（schema + size 制限）
- すべての更新系操作に `request_id` を必須化

3. Application Service Layer
- ユースケース単位（PR 作成、レビュー、マージ等）
- バリデーション、権限前提チェック、トランザクション制御

4. Integration Layer
- `GhAdapter`: `gh` コマンド標準実装
- `GitAdapter`: ローカル repo 操作（diff, log, checkout など）
- `ApiAdapter`: `gh api` 経由の REST/GraphQL 呼び出し

5. Infra Layer
- Job Queue（長時間処理の非同期実行）
- Cache（TTL + ETag）
- Audit Log（操作監査）

## バックエンド実装方針
- 実装順は Backend First（フロントエンドは後付け）
- バックエンドは feature-based module で分割し、機能単位で完結させる
- 各 feature module は「単体テスト完了 -> 結合」の順で統合する

## 主要設計原則
- Command Whitelist: 実行可能コマンドを事前定義し、任意シェル文字列を禁止
- JSON First: 可能な限り `--json` / API JSON を取得し、正規化して返す
- Idempotent Command API: 更新系 API は `request_id` で重複実行防止
- Deterministic Error Model: `auth`, `permission`, `validation`, `rate_limit`, `network`, `internal` に正規化
- Observability by Default: すべての操作に trace id を付与し、構造化ログに集約

## データフロー例（PR マージ）
1. UI が `POST /repos/{owner}/{repo}/pulls/{number}/merge` を実行
2. Application Service が事前条件を確認（status checks, 権限, merge method）
3. `GhAdapter` が `gh pr merge` を実行
4. 結果を Domain DTO に変換し SQLite キャッシュ更新
5. UI へ結果返却、失敗時は正規化エラーを返却

## API 境界（例）
- `GET /repos`
- `GET /repos/{owner}/{repo}`
- `GET /repos/{owner}/{repo}/pulls`
- `POST /repos/{owner}/{repo}/pulls`
- `POST /repos/{owner}/{repo}/pulls/{number}/review`
- `POST /repos/{owner}/{repo}/pulls/{number}/merge`
- `GET /repos/{owner}/{repo}/issues`
- `POST /repos/{owner}/{repo}/issues`
- `GET /repos/{owner}/{repo}/actions/runs`
- `POST /repos/{owner}/{repo}/actions/runs/{id}/rerun`

## 権限モデル
- `viewer`: 閲覧のみ
- `write`: issue/pr/release/actions 実行可
- `admin`: settings 系（webhooks, branch protection, collaborators）実行可

権限は API 呼び出し前に `gh api repos/{owner}/{repo}` の permission 情報で判定する。

## 不具合検知しやすさの設計
- すべてのコマンド実行を構造化ログ化（command_id, repo, duration_ms, exit_code）
- ユーザー向けエラーと内部原因を分離し、内部原因は error fingerprint で追跡
- 失敗が増えた操作を集計する lightweight metrics（成功率、P95、timeout率）
- panic/unhandled error は即時に共通ハンドラへ収束させる

## 開発分割
1. 共通基盤を先行実装
- `CommandRegistry`, `GhAdapter`, Error 正規化, Audit Log

2. feature-based に backend module を完了
- 例: `pull_requests` module を単体テスト完了後に統合

3. backend 完了後に frontend を後付け
- UI は backend 契約を呼ぶ薄い層に限定

4. E2E 優先
- 主要ユースケース（PR 作成/レビュー/マージ, Issue close, Workflow rerun）を自動化
