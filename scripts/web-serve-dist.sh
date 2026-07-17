#!/usr/bin/env bash
# Serve an already-built ./dist without rebuilding (fast).
# Default port 17880 — avoid clashing with common 8080 apps (other projects, Flutter, etc.).
# Override: ./scripts/web-serve-dist.sh 19000   or   PORT=19000 ./scripts/web-serve-dist.sh
set -euo pipefail
cd "$(dirname "$0")/../dist"
PORT="${1:-${PORT:-${RUSTY_PORT:-17880}}}"
echo "Serving RustyDasher at http://127.0.0.1:${PORT}/"
exec python3 - "$PORT" <<'PY'
import sys
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer

port = int(sys.argv[1])

class Handler(SimpleHTTPRequestHandler):
    def end_headers(self):
        path = self.path.split("?", 1)[0]
        if path.endswith((".html", ".js", "/")) or path in ("", "/"):
            self.send_header("Cache-Control", "no-store, no-cache, must-revalidate, max-age=0")
            self.send_header("Pragma", "no-cache")
        else:
            self.send_header("Cache-Control", "public, max-age=60")
        super().end_headers()

print(f"http://127.0.0.1:{port}/")
ThreadingHTTPServer(("127.0.0.1", port), Handler).serve_forever()
PY
