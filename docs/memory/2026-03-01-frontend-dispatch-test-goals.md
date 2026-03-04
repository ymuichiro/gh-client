# 2026-03-01 Frontend Dispatch Test Goals (Pre-Implementation)

## 背景
- 現状 backend は feature/service 実装は揃っているが、frontend が直接叩く IPC/dispatcher 境界が未実装。
- ここを先に固定しないと、フロント実装時に payload 仕様や権限処理で後戻りが発生する。

## 今回の対象
- `FrontendCommandEnvelope` を入口に、`command_id` + `payload` を既存 feature command handler にルーティングする backend API を実装。
- Tauri 依存を追加する前段として、crate 内でテスト可能な dispatcher をまず実装する。

## テスト要件
1. 契約整合
- `PAYLOAD_CONTRACT_VERSION` と envelope validation が機能すること。
- dispatcher が `STABLE_COMMAND_IDS` を取りこぼさないこと（コマンドサーフェス整合）。

2. 入力安全性
- `payload` の必須欠落/型不一致を `ValidationError` で返すこと。
- write/admin 操作で `permission` 不足時に `PermissionDenied` になること。

3. ルーティング正当性
- 代表 read/write コマンド（auth/repo/pr/issues/settings）で期待 command が実行されること。
- PR chat/diff 拡張 command が dispatcher から実行できること。

4. 失敗分類の継承
- dispatcher 経由でも既存 executor の失敗分類（AuthRequired/PermissionDenied/RateLimited/NetworkError）を壊さないこと。

5. レスポンス安定性
- 成功時レスポンスは JSON 化され、frontend で扱えること（DTO or `{ok:true}`）。

## 完了ゴール
- `cargo test` がグリーンで、少なくとも以下を満たす:
  - contract 整合テスト
  - dispatcher 単体テスト（入力/権限/主要ルート）
  - 既存 feature テスト回帰なし
- フロントは dispatcher 1入口（envelope）で全 stable command を呼び出せる状態になる。

## 実装メモ（事前決定）
- 権限未指定時は `viewer` 扱いにする（安全側デフォルト）。
- `repo.topics.replace` / `repo.branch.ref.get` のような内部 low-level command は `payload.args: string[]` で raw 実行できるフォールバックを用意。
- 破壊系 safety は既存 `safe_test_mode` に委譲する。
