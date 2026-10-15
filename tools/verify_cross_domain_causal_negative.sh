#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
OUT="${1:-artifacts/causal/cross_domain_causal_negative.jsonl}"
mkdir -p "$(dirname "$OUT")"

cargo run \
  --release \
  --manifest-path "$ROOT/crates/athlesia_integrated_cognitive_agent/Cargo.toml" \
  --example cross_domain_causal_negative \
  | tee "$OUT"

python3 - "$OUT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
records = [json.loads(line) for line in path.read_text().splitlines() if line.strip()]
assert len(records) == 1, records
record = records[0]
assert record == {
    "record": "negative_control",
    "case": "wrong_topology",
    "status": "REFUSED",
    "target_installed": False,
    "ledger_valid": True,
    "events": 2,
}, record
print(json.dumps({"verified": True, "case": record["case"], "status": record["status"], "artifact": str(path)}, separators=(",", ":")))
PY
