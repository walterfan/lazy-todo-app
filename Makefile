# Makefile for lazy-todo-app
#
# Convenience wrapper over npm / cargo / start.sh.
# Run `make help` for the full list of targets.

SHELL := /usr/bin/env bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c

# Override with e.g.  `make dev DB_DIR=~/my-data`
DB_DIR ?=

# Export only when provided, so the backend's default path still wins otherwise.
ifneq ($(strip $(DB_DIR)),)
export LAZY_TODO_DB_DIR := $(DB_DIR)
endif

NPM        := npm
CARGO      := cargo
TAURI_DIR  := src-tauri

.DEFAULT_GOAL := help
.PHONY: help install dev start build build-frontend preview check typecheck \
        rust-check lint test clean clean-frontend clean-rust clean-node \
        distclean release-tag pkb-check doctor

## ---------- Meta ----------

help: ## Show this help message
	@awk 'BEGIN {FS = ":.*?## "; printf "\nUsage: make <target>\n\nTargets:\n"} \
		/^[a-zA-Z_-]+:.*?## / { printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2 } \
		/^## / { printf "\n\033[1m%s\033[0m\n", substr($$0, 4) }' $(MAKEFILE_LIST)

doctor: ## Verify required toolchain is installed
	@command -v node  >/dev/null || { echo "✗ Node.js not found — https://nodejs.org/"; exit 1; }
	@command -v $(NPM)  >/dev/null || { echo "✗ npm not found"; exit 1; }
	@command -v $(CARGO) >/dev/null || { echo "✗ Rust/cargo not found — https://rustup.rs/"; exit 1; }
	@echo "✓ node  $$(node --version)"
	@echo "✓ npm   $$($(NPM) --version)"
	@echo "✓ cargo $$($(CARGO) --version)"

## ---------- Dependencies ----------

install: ## Install frontend dependencies (npm install)
	$(NPM) install

node_modules: package.json package-lock.json
	$(NPM) install
	@touch node_modules

## ---------- Run ----------

dev: node_modules ## Run the desktop app in dev mode (tauri dev, hot reload)
	@[[ -n "$${LAZY_TODO_DB_DIR:-}" ]] && echo "  DB directory: $$LAZY_TODO_DB_DIR" || true
	$(NPM) run tauri dev

start: dev ## Alias for `dev`

preview: node_modules ## Build frontend and serve with vite preview
	$(NPM) run build
	$(NPM) run preview

## ---------- Build ----------

build: node_modules ## Build production desktop bundles (tauri build)
	$(NPM) run tauri build
	@echo "✓ Bundles are in $(TAURI_DIR)/target/release/bundle/"

build-frontend: node_modules ## Build only the frontend (tsc + vite build)
	$(NPM) run build

## ---------- Checks ----------

check: typecheck rust-check ## Run frontend typecheck and Rust cargo check

typecheck: node_modules ## TypeScript type-check without emitting
	npx tsc --noEmit

rust-check: ## Run `cargo check` in src-tauri
	cd $(TAURI_DIR) && $(CARGO) check

lint: typecheck ## Lint (currently: typecheck). Hook in eslint here if added.

test: ## Run backend Rust tests (frontend has no tests yet)
	cd $(TAURI_DIR) && $(CARGO) test

## ---------- Clean ----------

clean: clean-frontend clean-rust ## Remove build artifacts (keeps node_modules)

clean-frontend: ## Remove dist/ (Vite output)
	rm -rf dist

clean-rust: ## Remove src-tauri/target/
	rm -rf $(TAURI_DIR)/target

clean-node: ## Remove node_modules/
	rm -rf node_modules

distclean: clean clean-node ## Remove everything rebuildable: dist, target, node_modules

## ---------- Misc ----------

release-tag: ## Run release tagging script (scripts/release_version.sh)
	$(NPM) run release:tag

pkb-check: ## Check Project Knowledge Base staleness
	$(NPM) run pkb:check
