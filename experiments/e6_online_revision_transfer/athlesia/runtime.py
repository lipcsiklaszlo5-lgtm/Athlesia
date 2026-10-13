"""Linux resource limits and measurements; no invented hardware numbers."""
from __future__ import annotations

import os
import resource
import sys

PROCESS_AS_LIMIT = 64 * 1024 * 1024


def constrain() -> dict[str, int | bool]:
    affinity = False
    if hasattr(os, "sched_getaffinity"):
        allowed = os.sched_getaffinity(0)
        os.sched_setaffinity(0, {min(allowed)})
        affinity = len(os.sched_getaffinity(0)) == 1
    enforced = False
    if sys.platform.startswith("linux"):
        resource.setrlimit(resource.RLIMIT_AS, (PROCESS_AS_LIMIT, PROCESS_AS_LIMIT))
        enforced = True
    return {"one_cpu_affinity": affinity, "address_space_limit_enforced": enforced,
            "address_space_limit_bytes": PROCESS_AS_LIMIT}


def peak_rss_bytes() -> int:
    raw = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    return int(raw if sys.platform == "darwin" else raw * 1024)
