#!/usr/bin/env bash
set -Eeuo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit 1

game_id="${ATHLESIA_ARC_GAME_ID:-wa30}"
steps="${ATHLESIA_ARC_MAX_STEPS:-200}"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
dir="${ATHLESIA_ARC_ARTIFACT_DIR:-artifacts/arc}"

mkdir -p "$dir"

export ATHLESIA_ARC_GAME_ID="$game_id"
export ATHLESIA_ARC_MAX_STEPS="$steps"
export ATHLESIA_AUTHORITY_AUDIT=1
export ATHLESIA_ARC_DIAG="$dir/${game_id}-${stamp}.diag.jsonl"
export ATHLESIA_ARC_SCORECARD="$dir/${game_id}-${stamp}.scorecard.json"
export ATHLESIA_ARC_TRACE="$dir/${game_id}-${stamp}.trace.jsonl"

cargo run \
  --manifest-path crates/athlesia_arc_agi_3_adapter/Cargo.toml \
  --bin athlesia_arc_public_smoke \
  2>&1 | tee "$dir/${game_id}-${stamp}.log"
