#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRATE="$ROOT/crates/athlesia_causal_wasm"
OUT="$ROOT/demo/wasm/pkg"

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "wasm-pack is required. Install it with: cargo install wasm-pack --locked" >&2
  exit 2
fi

rustup target add wasm32-unknown-unknown >/dev/null
rm -rf "$OUT"
mkdir -p "$OUT"

wasm-pack build "$CRATE" \
  --target web \
  --release \
  --out-dir "$OUT" \
  --no-typescript

echo "WASM package written to $OUT"
echo "Serve demo/wasm over HTTP; file:// is blocked by browser module policies."
