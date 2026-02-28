# バックエンド設計（`gh` 実行基盤）

## 目的
- GUI からの操作を GitHub に安全かつ一貫した方式で反映する
- `gh` の使いやすさを活かしつつ、UI で扱いやすい API/エラーへ変換する

## コンポーネント
1. Command Registry
- 実行可能コマンドを宣言的に定義
- 入力スキーマ（必須/型/制約）を保持

2. Executor
- `tokio::process::Command` で引数配列実行（shell 展開禁止）
- timeout, retry, cancel をサポート
- stdout/stderr/exit code を構造化して返却

3. Parser / Mapper
- `gh --json` 出力を内部 DTO に正規化
- `gh api` の REST/GraphQL レスポンス差異を吸収

4. Policy Guard
- 実行前に権限と対象 repo 所有権をチェック
- 危険操作（delete/force push 相当）は確認トークン必須

5. Job Runner
- 長時間処理（ログ取得、大量同期）を非同期ジョブ化
- 進捗とキャンセル API を提供

## 推奨ディレクトリ構成（feature-based）
```text
src-tauri/
  src/
    core/
      command_registry/
      executor/
      policy_guard/
      observability/
      infra/
        queue/
        sqlite/
        logging/
    features/
      repositories/
        command.rs
        service.rs
        dto.rs
        tests/
      pull_requests/
        command.rs
        service.rs
        dto.rs
        tests/
      issues/
        command.rs
        service.rs
        dto.rs
        tests/
      actions/
        command.rs
        service.rs
        dto.rs
        tests/
      releases/
        command.rs
        service.rs
        dto.rs
        tests/
      settings/
        command.rs
        service.rs
        dto.rs
        tests/
```

各 feature module は command/service/dto/tests を自己完結で持ち、依存は `core` へ一方向に限定する。

## コマンド実行ルール
- ルール1: すべてのコマンドは Registry 経由で起動
- ルール2: 動的に組み立てるのは「引数値のみ」、サブコマンドは固定
- ルール3: JSON 出力がある場合は必ず JSON モードを利用
- ルール4: PII/token をログに残さない（自動マスキング）
- ルール5: 失敗時は再現可能な最小情報（command id, exit code, stderr digest）を記録

## 不具合の検知性を上げる標準
- すべてのリクエストに `trace_id` と `request_id` を付与
- 重要操作（merge/close/delete/settings変更）は audit event を必須記録
- `stderr` は全文保存せず digest と分類結果を保持（機密漏えい対策）
- エラー分類の未定義値を禁止し、`unknown` は CI で失敗させる

## 失敗分類
- `AUTH_REQUIRED`: `gh auth` 未ログイン/期限切れ
- `PERMISSION_DENIED`: repo 権限不足
- `NOT_FOUND`: repo/issue/pr が存在しない
- `VALIDATION_ERROR`: 入力不正（title 空など）
- `RATE_LIMITED`: API 制限到達
- `UPSTREAM_ERROR`: GitHub 側 5xx
- `EXECUTION_ERROR`: `gh` 実行失敗・タイムアウト

UI には上記コード + 人間向けメッセージ + 再試行可否を返す。

## キャッシュ戦略
- Read API は短TTL（5〜30秒）+ ETag
- 更新系成功時は対象キーを即時無効化
- 一覧表示は stale-while-revalidate

## 監査ログ
- 記録項目: timestamp, actor(local user), repo, action, parameters hash, result
- 除外項目: token, secret value, full patch/body（必要時は hash のみ）

## API 契約例
### PR 作成
- Endpoint: `POST /repos/{owner}/{repo}/pulls`
- Input: `title`, `head`, `base`, `body`, `draft`
- Backend: `gh pr create ... --json number,url,state`
- Output: `number`, `url`, `state`

### Workflow 再実行
- Endpoint: `POST /repos/{owner}/{repo}/actions/runs/{run_id}/rerun`
- Backend: `gh run rerun {run_id}`
- Output: `accepted=true`

## セキュリティ要件
- `gh` バイナリパスを固定または検証
- PATH 汚染対策（起動時検証）
- 外部入力をコマンド文字列連結しない
- destructive 操作は確認フラグ + UI 二段階確認
- Tauri の許可 API を最小化し、不要な command を公開しない
- アップデータ/配布物の署名検証を必須化

## テスト戦略
1. Unit
- Registry 検証、入力バリデーション、エラーマッピング
- 失敗分類の網羅チェック（exhaustive match）

2. Integration
- `gh` モック（golden JSON）で DTO 変換を検証

3. E2E
- テスト用 GitHub repo を用意し、主要操作を夜間実行

4. Security
- `cargo audit`, `cargo deny`, dependency ライセンスチェック
- 危険操作 API に対する権限回避テスト

## テスト実行ポリシー（承認済み方針）
- 原則: ダミーではなく、本番相当の実操作を最優先
- 例外: 削除系などの重要破壊操作はテスト時のみダミー化

### 実操作で実施する対象
- 作成/更新/一覧/詳細/コメント/レビュー/マージ/再実行/キャンセル
- 権限不足・入力不正・レート制限などの異常系

### ダミー化する対象（テスト時）
- `repo delete`, `release delete`, `branch delete`, `webhook delete` などの破壊操作
- 実装方法: `SAFE_TEST_MODE=true` の場合、Executor が対象 command_id を no-op adapter へルーティング
- 検証方法: 実行リクエスト、監査ログ、戻り値、権限チェックだけを検証

### モジュール統合ゲート
- 各 feature module は以下を満たすまで統合しない
- 単体テスト: 正常/異常/境界/権限パターンを網羅
- 契約テスト: `gh --json` フィールド互換を固定
- 実操作テスト: 専用テスト repo 上で主要ユースケースが成功
