SHELL := /bin/bash
.DEFAULT_GOAL := help

UI_DIR := ui
TAURI_DIR := src-tauri
VITE_HOST := 127.0.0.1
VITE_PORT := 5173

.PHONY: help install dev-web dev-web-tauri run-desktop run-desktop-debug run-desktop-release tauri-dev build build-backend build-ui \
        test test-rust test-ui test-e2e test-e2e-live test-e2e-write \
        playwright-install fmt lint

help: ## 利用可能なコマンド一覧を表示
	@grep -E '^[a-zA-Z0-9_.-]+:.*## ' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*## "}; {printf "%-20s %s\n", $$1, $$2}'

install: ## 依存をインストール（UI）
	npm --prefix $(UI_DIR) install

dev-web: ## Web UI を mock モードで起動
	VITE_EXECUTION_MODE=mock npm --prefix $(UI_DIR) run dev -- --host $(VITE_HOST) --port $(VITE_PORT)

dev-web-tauri: ## Web UI を tauri モードで起動（desktop debug 用）
	npm --prefix $(UI_DIR) run dev -- --host $(VITE_HOST) --port $(VITE_PORT)

run-desktop: ## Tauri デスクトップアプリを1コマンドで起動（dev server 自動起動）
	@set -euo pipefail; \
	DEV_LOG=$$(mktemp); \
	echo "Starting UI dev server on http://$(VITE_HOST):$(VITE_PORT) ..."; \
	npm --prefix $(UI_DIR) run dev -- --host $(VITE_HOST) --port $(VITE_PORT) > "$$DEV_LOG" 2>&1 & \
	DEV_PID=$$!; \
	cleanup() { kill "$$DEV_PID" >/dev/null 2>&1 || true; }; \
	trap cleanup EXIT INT TERM; \
	READY=0; \
	for _ in $$(seq 1 120); do \
		if curl -fsS "http://$(VITE_HOST):$(VITE_PORT)" >/dev/null 2>&1; then READY=1; break; fi; \
		sleep 0.25; \
	done; \
	if [ "$$READY" -ne 1 ]; then \
		echo "dev server 起動待ちでタイムアウトしました。ログ: $$DEV_LOG"; \
		tail -n 80 "$$DEV_LOG" || true; \
		exit 1; \
	fi; \
	echo "Dev server is ready. Launching desktop app..."; \
	cd $(TAURI_DIR) && cargo run --features desktop --bin gh-client-desktop

run-desktop-debug: ## Tauri デスクトップアプリを起動（debug / devUrl=127.0.0.1:5173 前提）
	cd $(TAURI_DIR) && cargo run --features desktop --bin gh-client-desktop

run-desktop-release: ## Tauri デスクトップアプリを standalone 起動（release / custom-protocol）
	$(MAKE) build-ui
	cd $(TAURI_DIR) && cargo run --release --features desktop-custom-protocol --bin gh-client-desktop

tauri-dev: ## cargo tauri dev で起動
	cargo tauri dev --features desktop

build: ## UI と backend をビルド
	$(MAKE) build-ui
	$(MAKE) build-backend

build-backend: ## backend (desktop binary) をビルド
	cd $(TAURI_DIR) && cargo build --features desktop-custom-protocol --bin gh-client-desktop

build-ui: ## UI をビルド
	npm --prefix $(UI_DIR) run build

test: ## 全テスト（Rust + UI + E2E mock）
	$(MAKE) test-rust
	$(MAKE) test-ui
	$(MAKE) test-e2e

test-rust: ## Rust テストを実行
	cargo test

test-ui: ## UI unit/integration テストを実行
	npm --prefix $(UI_DIR) run test

playwright-install: ## Playwright (chromium) をインストール
	npx --prefix $(UI_DIR) playwright install chromium

test-e2e: ## Playwright E2E（mock）
	npm --prefix $(UI_DIR) run e2e

test-e2e-live: ## Playwright E2E（live read; GH_TEST_OWNER/GH_TEST_REPO 未指定時は自動解決）
	@OWNER="${GH_TEST_OWNER}"; \
	if [ -z "$$OWNER" ]; then OWNER="$$(gh api user --jq .login)"; fi; \
	REPO="${GH_TEST_REPO}"; \
	if [ -z "$$REPO" ]; then REPO="$$(gh repo list "$$OWNER" --json name --limit 1 --jq '.[0].name')"; fi; \
	if [ -z "$$OWNER" ] || [ -z "$$REPO" ]; then \
		echo "GH_TEST_OWNER/GH_TEST_REPO の解決に失敗しました。gh auth status を確認してください。"; \
		exit 1; \
	fi; \
	echo "Using $$OWNER/$$REPO"; \
	GH_CLIENT_LIVE_TEST=1 GH_TEST_OWNER="$$OWNER" GH_TEST_REPO="$$REPO" npm --prefix $(UI_DIR) run e2e:live

test-e2e-write: ## Playwright E2E（live write opt-in; GH_TEST_OWNER/GH_TEST_REPO 未指定時は自動解決）
	@OWNER="${GH_TEST_OWNER}"; \
	if [ -z "$$OWNER" ]; then OWNER="$$(gh api user --jq .login)"; fi; \
	REPO="${GH_TEST_REPO}"; \
	if [ -z "$$REPO" ]; then REPO="$$(gh repo list "$$OWNER" --json name --limit 1 --jq '.[0].name')"; fi; \
	if [ -z "$$OWNER" ] || [ -z "$$REPO" ]; then \
		echo "GH_TEST_OWNER/GH_TEST_REPO の解決に失敗しました。gh auth status を確認してください。"; \
		exit 1; \
	fi; \
	echo "Using $$OWNER/$$REPO"; \
	GH_CLIENT_LIVE_TEST=1 GH_CLIENT_LIVE_WRITE_TEST=1 GH_TEST_OWNER="$$OWNER" GH_TEST_REPO="$$REPO" npm --prefix $(UI_DIR) run e2e:write

fmt: ## Rust format check
	cargo fmt --all --check

lint: ## Rust lint (clippy)
	cargo clippy --all-targets --all-features -- -D warnings
