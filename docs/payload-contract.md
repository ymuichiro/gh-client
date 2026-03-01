# Frontend Payload Contract (v1)

## 目的
- フロントエンドとバックエンド間の IPC/command payload を固定化し、互換性の崩れをテストで検知できるようにする。
- 本契約は `src-tauri/src/contract/mod.rs` でコード化され、単体テストで registry と整合検証される。

## バージョン
- 契約バージョン: `2026-03-01.v1`
- 定義定数: `PAYLOAD_CONTRACT_VERSION`

## リクエスト envelope
```json
{
  "contract_version": "2026-03-01.v1",
  "request_id": "req-123",
  "command_id": "repo.list",
  "permission": "viewer",
  "payload": {
    "owner": "octocat",
    "limit": 20
  }
}
```

### フィールド
- `contract_version`: 必須。現在は `2026-03-01.v1` のみ受け付ける。
- `request_id`: 必須。空文字不可。
- `command_id`: 必須。安定契約に含まれる command id のみ許可。
- `permission`: 任意。`viewer | write | admin`。
- `payload`: 任意オブジェクト。`command_id` ごとの入力を保持。

## 安定 command surface
- 安定コマンド一覧は `STABLE_COMMAND_IDS` で固定。
- `validate_registry_contract()` が default registry と完全一致を検証する。
- command の追加/削除/改名は契約変更として扱い、バージョン更新を要する。

## 互換性ポリシー
- 破壊的変更（command id 変更、必須フィールド変更）はバージョンを上げる。
- 後方互換な追加（新 command id 追加）は `v1` のまま許可しない。明示バージョン更新を行う。

## 実装参照
- コード: `src-tauri/src/contract/mod.rs`
- 検証テスト: `contract::tests::stable_contract_matches_default_registry`
