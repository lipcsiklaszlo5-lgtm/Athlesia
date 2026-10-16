#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CASES="${1:-128}"
MANIFEST="$ROOT/crates/athlesia_integrated_cognitive_agent/Cargo.toml"

cd "$ROOT"
cargo test --locked \
  --manifest-path "$MANIFEST" \
  --test discovered_context_causal_agent \
  -- --test-threads=1

cargo build --locked --release \
  --manifest-path "$MANIFEST" \
  --example discovered_context_bench

"$ROOT/crates/athlesia_integrated_cognitive_agent/target/release/examples/discovered_context_bench" \
  --cases "$CASES"
