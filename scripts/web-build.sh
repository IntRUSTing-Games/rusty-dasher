#!/usr/bin/env bash
# Produce a static site in ./dist for any static host (itch.io, Pages, etc.).
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="${HOME}/.local/bin:${HOME}/.cargo/bin:${PATH}"
env -u NO_COLOR trunk build --release --color never "$@"
# Avoid stale WASM/JS TypeErrors after hash-changing rebuilds.
"$(dirname "$0")/clear-firefox-cache.sh" || true
