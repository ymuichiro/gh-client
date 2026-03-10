# Release Readiness (2026-03-10)

## 1. 対象スコープ
- 画面構成の簡素化: `Issues / Pull Requests / Settings` の3ページ構成
- リポジトリ対象設定の一本化: 設定画面に統合
- Issue/PR 一覧と詳細の分離（詳細はモーダル）
- ローディング表示の導入とデータ取得体感の改善
- Issue/PR キャッシュ運用（更新ボタン + 定期更新フロー）
- PR Diff のシンタックスハイライト + Inline/Split切替
- 外部ブラウザ起動修正（Tauri opener）
- Diff閲覧性改善（行単位スクロール廃止、全体スクロール化）
- 詳細モーダルの拡張（PC広幅 + モバイル下部シート）
- Inboxレイアウトの全幅化（左寄り表示の解消）

## 2. 不要物チェック
- `git ls-files --others --exclude-standard` で、今回実装に関係ない生成物や一時ファイルは検出なし
- 差分は `src-tauri` と `ui/src`、`ui` の依存更新、`docs` 追加のみ

## 3. 実行済み検証
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS
- `npm --prefix ui run test -- --run`: PASS
- `npm --prefix ui run build`: PASS
- `npm --prefix ui run e2e`: PASS

## 4. 既知事項（非ブロッカー）
- UI build 時に chunk size 警告あり（機能影響なし）
- Playwright 実行時に `NO_COLOR/FORCE_COLOR` の警告あり（機能影響なし）

## 5. ブルーテスト前の手動確認項目
- Issues / Pull Requests 画面が横幅いっぱいに広がること
- PR Diff が行単位ではなく全体コンテナで横スクロールできること（スクロールバー非表示）
- Issue/PR の `GitHubで開く` で OS 既定ブラウザが起動すること
- PR/Issue 詳細モーダルが PC で十分広く、モバイル幅では下部シート風に表示されること
- Settings で対象リポジトリ選択が完結すること

## 6. コミット方針
- 今回の差分を1コミットにまとめる
- コミット後に `origin/codex/issues-pr-settings-split` へ push
