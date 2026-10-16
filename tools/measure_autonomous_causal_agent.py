"""Measure one native benchmark child, pinned to one allowed Linux CPU.

No Gradio, browser, cargo build or parent interpreter is counted as solver RAM.
Conversely this is the WHOLE native harness, not an isolated inner-kernel timer.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import resource
import subprocess
import time


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--cases", type=int, default=128)
    parser.add_argument(
        "--multi-effects",
        action="store_true",
        help="run the explicit A2 two-effect grammar instead of the A1 default",
    )
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    if not 1 <= args.cases <= 10000:
        parser.error("cases must be in 1..10000")
    binary = args.binary.resolve(strict=True)
    if platform.system() != "Linux" or not hasattr(os, "sched_setaffinity"):
        raise SystemExit("Single-CPU/RSS verification currently requires Linux.")
    cpu = min(os.sched_getaffinity(0))
    os.sched_setaffinity(0, {cpu})
    if os.sched_getaffinity(0) != {cpu}:
        raise SystemExit("Single-CPU affinity was not enforced.")

    # This fresh Python process starts exactly one child. RUSAGE_CHILDREN's
    # maximum therefore belongs to that binary, never to a previous cargo run.
    start = time.perf_counter_ns()
    command = [str(binary), "--cases", str(args.cases)]
    if args.multi_effects:
        command.append("--multi-effects")
    result = subprocess.run(
        command,
        text=True, capture_output=True, check=True, timeout=120,
    )
    wall_ms = (time.perf_counter_ns() - start) / 1_000_000
    usage = resource.getrusage(resource.RUSAGE_CHILDREN)
    records = [json.loads(line) for line in result.stdout.splitlines() if line.strip()]
    summaries = [record for record in records if record.get("record") == "benchmark"]
    if len(summaries) != 1:
        raise SystemExit("Missing or duplicate benchmark summary.")
    summary = summaries[0]
    for field in ("cases", "cold_solved", "warm_solved", "transfer_solved"):
        if summary.get(field) != args.cases:
            raise SystemExit(f"Unverified result: {field}")
    peak_bytes = usage.ru_maxrss * 1024  # Linux reports KiB.
    if peak_bytes >= 250_000_000:
        raise SystemExit(f"250 MB native process memory budget exceeded: {peak_bytes}")
    with binary.open("rb") as stream:
        binary_sha = hashlib.file_digest(stream, "sha256").hexdigest()
    report = {
        "verified": True,
        "binary_sha256": binary_sha,
        "platform": platform.platform(),
        "single_cpu_affinity": sorted(os.sched_getaffinity(0)),
        "whole_child_cpu_ms": (usage.ru_utime + usage.ru_stime) * 1000,
        "whole_child_wall_ms_including_startup": wall_ms,
        "whole_child_peak_rss_bytes": peak_bytes,
        "scope": "Native fixture+agent+memory replay+transfer; not inner-kernel CPU latency.",
        "result": summary,
        "sample": next((r for r in records if r.get("record") == "sample"), None),
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
