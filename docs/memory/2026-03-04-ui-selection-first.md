# 2026-03-04 ui-selection-first

## 何を変更したか
- backend に `auth.organizations.list` を追加し、`gh api user/orgs` から organization 候補を取得可能にした。
- payload contract を `2026-03-04.v3` に更新し、stable command に `auth.organizations.list` を追加。
- フロントエンドの対象リポジトリ選択を設定画面フローに集約。
- コマンドフォームの `owner/repo` 候補を選択式へ変更し、可能な項目（branch/tag/PR番号/Issue番号/run_id）を動的候補選択化。
- E2E をリポジトリ選択フロー前提に更新。

## なぜ変更したか
- GitHub Web UI と同様に、組織とリポジトリを画面上で選択できる操作感へ寄せるため。
- 手入力起因の typo や対象 repo 誤指定を減らし、操作ミスを防ぐため。

## 検証
- `cargo test --manifest-path src-tauri/Cargo.toml`（backend 全テスト）: pass
- `npm --prefix ui run test`: pass
- `npm --prefix ui run build`: pass
- `npm --prefix ui run e2e`（mock read）: pass

## 発見した課題
- owner/repo 候補は `gh` セッションの権限に依存するため、権限外 repo は候補に出ない。
- 動的候補 API は read コマンド依存のため、権限不足時は空候補になる（実行自体は従来どおり可能）。

## 次のアクション
- 必要に応じて候補キャッシュ（localStorage）を導入し、起動直後の候補表示速度を改善。
- command ごとの動的候補（hook_id/key_id/comment_id 等）を段階的に拡張。
