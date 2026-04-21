#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS] [MODE]

Modes:
  dev       Start in development mode with hot reload (default)
  build     Build production bundles (frontend + Tauri app)
  preview   Preview the frontend production build in browser

Options:
  --db-dir PATH   Set custom database directory (LAZY_TODO_DB_DIR)
  --check         Run type checks only (tsc + cargo check)
  -h, --help      Show this help message

Examples:
  ./start.sh                         # dev mode
  ./start.sh dev                     # dev mode (explicit)
  ./start.sh build                   # production build
  ./start.sh --db-dir ~/my-data dev  # dev mode with custom DB path
  ./start.sh --check                 # type-check frontend + backend
EOF
    exit 0
}

MODE="dev"
RUN_CHECK=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            usage
            ;;
        --db-dir)
            export LAZY_TODO_DB_DIR="$2"
            shift 2
            ;;
        --check)
            RUN_CHECK=true
            shift
            ;;
        dev|build|preview)
            MODE="$1"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            usage
            ;;
    esac
done

if ! command -v node &>/dev/null; then
    echo "Error: Node.js is not installed. See https://nodejs.org/"
    exit 1
fi

if ! command -v cargo &>/dev/null; then
    echo "Error: Rust is not installed. See https://rustup.rs/"
    exit 1
fi

if [[ ! -d node_modules ]]; then
    echo "Installing frontend dependencies..."
    npm install
fi

if [[ "$RUN_CHECK" == true ]]; then
    echo "==> TypeScript check..."
    npx tsc --noEmit
    echo "==> Rust check..."
    (cd src-tauri && cargo check)
    echo "All checks passed."
    exit 0
fi

case "$MODE" in
    dev)
        echo "Starting in development mode..."
        [[ -n "${LAZY_TODO_DB_DIR:-}" ]] && echo "  DB directory: $LAZY_TODO_DB_DIR"
        npm run tauri dev
        ;;
    build)
        echo "Building production bundles..."
        npm run tauri build
        echo "Done. Bundles are in src-tauri/target/release/bundle/"
        ;;
    preview)
        echo "Building frontend and starting preview server..."
        npm run build
        npm run preview
        ;;
esac
