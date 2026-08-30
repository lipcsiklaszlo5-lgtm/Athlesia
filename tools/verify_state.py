from pathlib import Path
import hashlib
import json
import sys

ROOT = Path(__file__).resolve().parent.parent
STATE_FILE = ROOT / "state" / "project_state.json"

def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main() -> int:
    if not STATE_FILE.is_file():
        print("STATE VERIFY: FAIL")
        print("Missing state/project_state.json")
        return 1

    state = json.loads(STATE_FILE.read_text(encoding="utf-8"))
    manifest = state["implementation_sha256"]

    failures = []

    for relative, expected in manifest.items():
        path = ROOT / relative

        if not path.is_file():
            failures.append(f"MISSING {relative}")
            continue

        actual = sha256(path)

        if actual != expected:
            failures.append(f"DRIFT {relative}")

    if failures:
        print("STATE VERIFY: DRIFT DETECTED")
        for failure in failures:
            print(f"  {failure}")
        return 1

    print("STATE VERIFY: PASS")
    print(f"Rust stage: {state['rust_port']['stage']}")
    print(f"Rust status: {state['rust_port']['status']}")
    print(f"Next layer: {state['rust_port']['next_layer']}")
    print(f"Tracked implementation files: {len(manifest)}")
    return 0

if __name__ == "__main__":
    sys.exit(main())
