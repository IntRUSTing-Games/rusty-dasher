#!/usr/bin/env bash
# Produce a static site in ./dist for any static host (itch.io, Pages, etc.).
#
# Local default: cargo profile **wasm-fast** (parallel codegen, no LTO) — much
# quicker rebuilds. Production (GitHub Actions) still runs:
#   trunk build --release
# and is unaffected by this default.
#
# Usage:
#   ./scripts/web-build.sh              # local fast (wasm-fast)
#   ./scripts/web-build.sh --release    # production-like (release + thin LTO)
#   WEB_BUILD_RELEASE=1 ./scripts/web-build.sh
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="${HOME}/.local/bin:${HOME}/.cargo/bin:${PATH}"

RELEASE=0
ARGS=()
for arg in "$@"; do
  case "$arg" in
    --release|-r)
      RELEASE=1
      ;;
    *)
      ARGS+=("$arg")
      ;;
  esac
done

if [[ "${WEB_BUILD_RELEASE:-0}" == "1" || "${WEB_BUILD_RELEASE:-}" == "true" ]]; then
  RELEASE=1
fi

# Prefer multi-core cargo; never force -j1. Respect an explicit CARGO_BUILD_JOBS.
if [[ -z "${CARGO_BUILD_JOBS:-}" ]]; then
  if command -v nproc >/dev/null 2>&1; then
    export CARGO_BUILD_JOBS="$(nproc)"
  fi
fi

# Incremental helps local wasm-fast; CI sets CARGO_INCREMENTAL=0 itself.
if [[ "$RELEASE" -eq 0 ]]; then
  export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-1}"
  echo "web-build: local profile wasm-fast (jobs=${CARGO_BUILD_JOBS:-default})"
  env -u NO_COLOR trunk build --cargo-profile wasm-fast --color never "${ARGS[@]+"${ARGS[@]}"}"
else
  echo "web-build: production profile release (jobs=${CARGO_BUILD_JOBS:-default})"
  env -u NO_COLOR trunk build --release --color never "${ARGS[@]+"${ARGS[@]}"}"
fi

# Avoid stale WASM/JS TypeErrors after hash-changing rebuilds.
"$(dirname "$0")/clear-firefox-cache.sh" || true
