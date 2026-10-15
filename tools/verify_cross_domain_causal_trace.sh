#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
OUT="${1:-artifacts/causal/cross_domain_causal_trace.jsonl}"
mkdir -p "$(dirname "$OUT")"

cargo run \
  --manifest-path "$ROOT/crates/athlesia_integrated_cognitive_agent/Cargo.toml" \
  --example cross_domain_causal_trace \
  | tee "$OUT"

python3 - "$OUT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
records = [json.loads(line) for line in path.read_text().splitlines() if line.strip()]

stages = [r for r in records if r.get("record") == "stage"]
ledgers = [r for r in records if r.get("record") == "ledger"]
summaries = [r for r in records if r.get("record") == "summary"]

expected_stages = [
    "OBSERVE",
    "ABSTRACT",
    "TRANSFER",
    "AUTHORITY",
    "TARGET_TRIAL",
    "FALSIFY",
    "AUTHORITY",
    "ACTIVE_PROBE",
    "REVISE",
]
actual_stages = [r.get("stage") for r in stages]
assert actual_stages == expected_stages, (actual_stages, expected_stages)
assert len(ledgers) == 7, len(ledgers)
assert len(summaries) == 1, len(summaries)

for expected_sequence, record in enumerate(ledgers):
    assert record["sequence"] == expected_sequence, record
    assert record["record_hash"] != 0, record
    if expected_sequence == 0:
        assert record["previous_hash"] == 0, record
    else:
        assert record["previous_hash"] == ledgers[expected_sequence - 1]["record_hash"], record

summary = summaries[0]
assert summary["status"] == "REVISED", summary
assert summary["epoch"] == 1, summary
assert summary["ledger_valid"] is True, summary
assert summary["events"] == len(ledgers), summary
assert isinstance(summary["elapsed_ns"], int) and summary["elapsed_ns"] > 0, summary

print(json.dumps({
    "verified": True,
    "stage_count": len(stages),
    "ledger_events": len(ledgers),
    "final_status": summary["status"],
    "epoch": summary["epoch"],
    "elapsed_ns": summary["elapsed_ns"],
    "artifact": str(path),
}, separators=(",", ":")))
PY
