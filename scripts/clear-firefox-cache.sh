#!/usr/bin/env bash
# Drop Firefox's HTTP disk cache so local rebuilds (new WASM hashes) load cleanly.
# Safe to run while Firefox is open; it recreates the dirs as needed.
set -euo pipefail

cleared=0
for dir in \
  "${HOME}/.cache/mozilla/firefox"/*/cache2 \
  "${HOME}/.mozilla/firefox"/*/cache2 \
  "${HOME}/snap/firefox/common/.cache/mozilla/firefox"/*/cache2 \
  "${HOME}/.var/app/org.mozilla.firefox/.cache/mozilla/firefox"/*/cache2
do
  if [ -d "$dir" ]; then
    # entries/ holds the bulk of cached responses
    rm -rf "${dir}/entries" "${dir}/doomed" 2>/dev/null || true
    mkdir -p "${dir}/entries" 2>/dev/null || true
    cleared=$((cleared + 1))
    echo "cleared Firefox cache: $dir"
  fi
done

# Occasionally holds stale compiled bits
for dir in "${HOME}/.cache/mozilla/firefox"/*/startupCache; do
  if [ -d "$dir" ]; then
    rm -rf "${dir:?}"/* 2>/dev/null || true
    echo "cleared startupCache: $dir"
  fi
done

if [ "$cleared" -eq 0 ]; then
  echo "no Firefox cache2 dirs found (ok if you only use Chrome)"
else
  echo "Firefox disk cache cleared ($cleared profile(s)). Hard-refresh still helps if a tab is mid-load."
fi
