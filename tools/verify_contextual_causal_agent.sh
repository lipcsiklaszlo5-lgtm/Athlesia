#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_dir"
manifest="crates/athlesia_integrated_cognitive_agent/Cargo.toml"
cases="${1:-128}"
if ! [[ "$cases" =~ ^[0-9]+$ ]] || (( cases < 1 || cases > 10000 )); then
  echo "usage: $0 [cases 1..10000]" >&2
  exit 2
fi

cargo test --locked --manifest-path "$manifest" \
  --test contextual_causal_agent -- --test-threads=1
cargo build --locked --release --manifest-path "$manifest" \
  --example contextual_causal_bench
mkdir -p artifacts/autonomous_causal_agent
cargo run --locked --release --manifest-path "$manifest" \
  --example contextual_causal_bench -- --cases "$cases" \
  | tee "artifacts/autonomous_causal_agent/contextual.jsonl"
