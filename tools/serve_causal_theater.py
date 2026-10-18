#!/usr/bin/env python3
"""Serve the existing demo tree and route legacy entrypoints to Causal Theater."""
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit
import argparse
import os


class TheaterHandler(SimpleHTTPRequestHandler):
    def do_GET(self):
        url = urlsplit(self.path)
        if url.path in {"/", "/index.html", "/wasm", "/wasm/", "/wasm/index.html"}:
            self.send_response(302)
            self.send_header("Location", "/causal-theater/" + ("?" + url.query if url.query else ""))
            self.send_header("Cache-Control", "no-store")
            self.end_headers()
            return
        super().do_GET()

    def end_headers(self):
        self.send_header("Cache-Control", "no-store")
        super().end_headers()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=8081)
    args = parser.parse_args()
    directory = Path(__file__).resolve().parents[1] / "demo"
    handler = partial(TheaterHandler, directory=str(directory))
    server = ThreadingHTTPServer(("0.0.0.0", args.port), handler)
    if os.environ.get("CODESPACE_NAME"):
        host = os.environ.get("GITHUB_CODESPACES_PORT_FORWARDING_DOMAIN", "app.github.dev")
        print(f"Open: https://{os.environ['CODESPACE_NAME']}-{args.port}.{host}/causal-theater/", flush=True)
    else:
        print(f"Open: http://127.0.0.1:{args.port}/causal-theater/", flush=True)
    print("Legacy root and /wasm/ entries redirect to the current demo.", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        server.server_close()
