#!/usr/bin/env bash
# Serve the game in a browser via Trunk (WASM).
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="${HOME}/.local/bin:${HOME}/.cargo/bin:${PATH}"
# Trunk chokes if NO_COLOR=1 is set as a bare flag value
exec env -u NO_COLOR trunk serve --color never "$@"
