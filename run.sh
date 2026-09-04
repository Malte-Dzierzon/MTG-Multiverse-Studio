#!/usr/bin/env bash
# mtg-tui — Rust/ratatui Build & Start
# Usage: mtg-tui [--db /path/to/db]
set -euo pipefail

# Symlinks auflösen, damit der globale Befehl funktioniert
SCRIPT="$(readlink -f "${BASH_SOURCE[0]}")"
cd "$(dirname "$SCRIPT")"

if ! command -v cargo >/dev/null; then
    echo "✗ cargo not found — install rustup first"
    exit 1
fi

cargo build --release --quiet
exec ./target/release/mtg-tui "$@"
