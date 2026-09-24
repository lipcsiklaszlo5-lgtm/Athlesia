#!/usr/bin/env bash
# One-command, terminal-safe reproduction of the Open Intelligence evidence.
set -Eeuo pipefail

repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd -- "$repo_root"

demo_dir=experiments/open_intelligence_core_demo
result_dir=artifacts/open_intelligence
result_file="$result_dir/result.json"

printf '1/4 Syntax check\n'
python3 -B -c 'import ast, pathlib, sys; [ast.parse(pathlib.Path(name).read_text(), filename=name) for name in sys.argv[1:]]' \
    "$demo_dir/core_demo.py" "$demo_dir/test_core_demo.py" "$demo_dir/showcase.py"

printf '2/4 Deterministic acceptance tests\n'
PYTHONPATH="$demo_dir" python3 -B -m unittest discover \
    --start-directory "$demo_dir" \
    --pattern 'test_*.py' \
    --verbose

printf '3/4 Live transfer showcase\n'
python3 -B "$demo_dir/showcase.py"

printf '4/4 Full held-out benchmark\n'
mkdir -p -- "$result_dir"
python3 -B "$demo_dir/core_demo.py" --json-out "$result_file"

printf '\nDONE: %s\n' "$result_file"
printf 'The shell remains open; this script does not call exit or replace the terminal.\n'
