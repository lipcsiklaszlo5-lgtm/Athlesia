#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_dir"
manifest="crates/athlesia_integrated_cognitive_agent/Cargo.toml"

# A named integration target cannot silently succeed with zero matching tests.
cargo test --locked --manifest-path "$manifest" --test autonomous_causal_agent -- --test-threads=1
cargo test --locked --manifest-path "$manifest" --lib cross_domain_causal_core -- --test-threads=1
cargo build --locked --release --manifest-path "$manifest" --example autonomous_causal_bench
python3 tools/measure_autonomous_causal_agent.py \
  --binary crates/athlesia_integrated_cognitive_agent/target/release/examples/autonomous_causal_bench \
  --cases "${1:-128}" \
  --output artifacts/autonomous_causal_agent/native.json
