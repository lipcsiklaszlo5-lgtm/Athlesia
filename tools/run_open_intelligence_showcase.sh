#!/usr/bin/env bash
# Fast, visible transfer demonstration using a seed range separate from scoring.
set -Eeuo pipefail

repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd -- "$repo_root"

python3 -B experiments/open_intelligence_core_demo/showcase.py
