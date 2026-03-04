# 2026-03-04 makefile-commands

## 何を変更したか
- ルートに `Makefile` を追加し、起動・ビルド・テストで日常的に使うコマンドを集約。
- `README.md` の「ローカル起動」節に `make help` への導線を追加。

## なぜ変更したか
- 実行コマンドが README 内で分散していたため、起動手順とテスト手順を短いターゲット名で統一して操作ミスを減らすため。

## 追加したターゲット
- `install`, `dev-web`, `dev-web-tauri`, `run-desktop`, `run-desktop-debug`, `tauri-dev`
- `build`, `build-ui`, `build-backend`
- `test`, `test-rust`, `test-ui`, `test-e2e`
- `test-e2e-live`, `test-e2e-write`（`GH_TEST_OWNER`/`GH_TEST_REPO` 未指定時は `gh` から自動解決）
- `playwright-install`, `fmt`, `lint`

## 検証
- `make help` を実行したが、ローカル環境の Xcode license 未同意エラーで実行不可。
- `Makefile` の内容（ターゲット定義/構文）を静的確認した。

## 発見した課題
- `gh` 未ログイン時は live 系ターゲットの owner/repo 自動解決に失敗する。`gh auth status` で事前確認が必要。

## 次のアクション
- CI で `make test` / `make lint` を利用するか検討。

## 追記: white screen 調査（run-desktop）
- `cargo run --manifest-path src-tauri/Cargo.toml --features desktop --bin gh-client-desktop`（debug）で起動したアプリは、`tauri.conf.json` の `devUrl`（`http://127.0.0.1:5173`）へアクセスする。
- 実測として、`python3 -m http.server 5173` を立てた状態で debug 起動すると `GET /` が記録され、`devUrl` 参照が確認できた。
- そのため `dev-web` 未起動の debug 実行では WebView が読み込めず白画面になる。

## 追記: 対策
- `Makefile` の `run-desktop` を release 起動へ変更し、毎回 `build-ui` 後に `frontendDist` を使って起動するよう修正。
- debug 実行は `run-desktop-debug` に分離し、`dev-web-tauri` 先行起動が必要であることを README に明記。

## 追記: white screen の追加原因（release run）
- `target/release/gh-client-desktop` の文字列を確認すると `../ui/dist` がそのまま埋め込まれていた。
- `make run-desktop` はリポジトリルートで `cargo run --manifest-path src-tauri/Cargo.toml` を実行していたため、相対 `../ui/dist` が実行時 CWD 基準で解決され、`/Users/you/github/ymuichiro/ui/dist` 側を見に行く可能性がある。
- 実パスは `src-tauri` 基準でないと正しく解決されないため、画面白化の再現条件と整合する。

## 追記: 恒久対策（Makefile）
- `run-desktop` / `run-desktop-debug` / `build-backend` を `cd src-tauri && cargo ...` に変更。
- これにより `tauri.conf.json` の相対パス（`../ui/dist`）が想定どおり `gh-client/ui/dist` を指す。

## 追記: 白画面耐性の追加修正（Vite base）
- `ui/vite.config.ts` に `base: "./"` を追加し、ビルド成果物のアセット参照を `/assets/...` から `./assets/...` に変更。
- `ui/dist/index.html` で相対パスになったことを確認。
- これにより Tauri の `frontendDist` 読み込み時に絶対パス解決が失敗する環境差を回避する。

## 追記: 根本原因の確定（Tauri dev 判定）
- `tauri-build` / `tauri` crate の実装を確認し、`cfg(dev)` は `custom-protocol` feature が無いと有効になる仕様を確認。
- `cargo run --release --features desktop` は release でも `cfg(dev)` 扱いになり、`devUrl` を参照する。
- 実測でも release 実行時に `127.0.0.1:5173` へ `GET /` が発生し、dev server 未起動時は白画面になることを再確認。

## 追記: 起動設計の変更（1コマンド化）
- `make run-desktop` を「dev server 自動起動 + 起動待ち + desktop app 起動」に変更し、単発実行で白画面にならないよう修正。
- 起動待ちは `curl` で `127.0.0.1:5173` をポーリングし、ready 後に `cargo run --features desktop` を実行。
- standalone release 経路は `make run-desktop-release` として分離。
