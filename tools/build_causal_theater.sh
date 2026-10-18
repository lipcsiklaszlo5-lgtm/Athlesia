#!/usr/bin/env bash
# Build the EXISTING Rust/WASM bridge and identify its generated executable.
set -euo pipefail

theater_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
theater_output="$theater_root/demo/wasm/pkg"

if ! command -v wasm-pack >/dev/null 2>&1; then
  printf 'wasm-pack is required. Install once: cargo install wasm-pack --locked\n' >&2
  exit 2
fi
if ! command -v node >/dev/null 2>&1; then
  printf 'Node.js is required to write the executable provenance manifest.\n' >&2
  exit 2
fi

rustup target add wasm32-unknown-unknown
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_RELEASE_DEBUG=0 CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
  wasm-pack build "$theater_root/crates/athlesia_causal_wasm" \
  --target web --release --out-dir "$theater_output" --no-typescript

node "$theater_root/tools/write_theater_manifest.mjs" "$theater_root" "$theater_output"
printf 'Serve: python3 -m http.server 8080 --bind 0.0.0.0 --directory demo\n'
printf 'Open:  /causal-theater/ on the forwarded port\n'
