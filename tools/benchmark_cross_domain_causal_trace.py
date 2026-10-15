#!/usr/bin/env python3
"""Repeat the production causal-trace example and verify every run."""
from __future__ import annotations

import argparse
import json
import math
import pathlib
import subprocess
import sys
from statistics import median

EXPECTED_STAGES = [
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


def percentile(sorted_values: list[int], fraction: float) -> int:
    index = max(0, min(len(sorted_values) - 1, math.ceil(fraction * len(sorted_values)) - 1))
    return sorted_values[index]


def verify_run(stdout: str) -> int:
    records = [json.loads(line) for line in stdout.splitlines() if line.strip()]
    stages = [record for record in records if record.get("record") == "stage"]
    ledgers = [record for record in records if record.get("record") == "ledger"]
    summaries = [record for record in records if record.get("record") == "summary"]

    assert [record.get("stage") for record in stages] == EXPECTED_STAGES
    assert len(ledgers) == 7
    assert len(summaries) == 1
    for expected_sequence, record in enumerate(ledgers):
        assert record["sequence"] == expected_sequence
        assert record["record_hash"] != 0
        expected_previous = 0 if expected_sequence == 0 else ledgers[expected_sequence - 1]["record_hash"]
        assert record["previous_hash"] == expected_previous

    summary = summaries[0]
    assert summary["status"] == "REVISED"
    assert summary["epoch"] == 1
    assert summary["ledger_valid"] is True
    assert summary["events"] == len(ledgers)
    elapsed_ns = summary["elapsed_ns"]
    assert isinstance(elapsed_ns, int) and elapsed_ns > 0
    return elapsed_ns


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runs", type=int, default=100)
    parser.add_argument("--release", action="store_true", help="benchmark the optimized binary")
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args()
    if args.runs < 2:
        parser.error("--runs must be at least 2")

    root = pathlib.Path(
        subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True).strip()
    )
    crate_manifest = root / "crates/athlesia_integrated_cognitive_agent/Cargo.toml"
    profile = "release" if args.release else "debug"
    build_args = ["cargo", "build", "--quiet", "--manifest-path", str(crate_manifest), "--example", "cross_domain_causal_trace"]
    if args.release:
        build_args.insert(2, "--release")
    subprocess.run(build_args, check=True)

    binary = crate_manifest.parent / "target" / profile / "examples" / "cross_domain_causal_trace"
    if not binary.is_file():
        raise SystemExit(f"built example not found: {binary}")

    elapsed = []
    for run_index in range(args.runs):
        completed = subprocess.run([str(binary)], check=True, capture_output=True, text=True)
        elapsed.append(verify_run(completed.stdout))
        if (run_index + 1) % 10 == 0 or run_index + 1 == args.runs:
            print(f"verified {run_index + 1}/{args.runs}", file=sys.stderr)

    ordered = sorted(elapsed)
    report = {
        "benchmark": "Athlesia bounded cross-domain causal trace",
        "profile": profile,
        "runs": args.runs,
        "successes": len(elapsed),
        "failures": 0,
        "trace_stages": len(EXPECTED_STAGES),
        "ledger_events": 7,
        "elapsed_ns_min": ordered[0],
        "elapsed_ns_p50": int(median(ordered)),
        "elapsed_ns_p95": percentile(ordered, 0.95),
        "elapsed_ns_max": ordered[-1],
        "binary": str(binary),
        "scope": "fixed bounded causal fixture; runner-reported example interval including trace emission",
    }
    encoded = json.dumps(report, indent=2)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n")
    print(encoded)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
