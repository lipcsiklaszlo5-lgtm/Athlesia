#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
manifest="$repo_root/crates/athlesia_integrated_cognitive_agent/Cargo.toml"

cargo test --locked --manifest-path "$manifest" --test active_context_agent -- --test-threads=1
cargo run --locked --manifest-path "$manifest" --example active_context_bench
