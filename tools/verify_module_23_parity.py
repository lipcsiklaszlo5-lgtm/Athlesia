from pathlib import Path
import hashlib
import json
import sys

ROOT = Path(__file__).resolve().parents[1]
PARITY_FILE = ROOT / "state" / "module_23_parity.json"

def sha256(path):
    return hashlib.sha256(
        path.read_bytes()
    ).hexdigest()

if not PARITY_FILE.exists():
    print("MODULE 23 PARITY VERIFY: FAIL")
    print("Missing parity manifest.")
    sys.exit(1)

manifest = json.loads(
    PARITY_FILE.read_text(
        encoding="utf-8"
    )
)

errors = []

if manifest.get("status") != "frozen":
    errors.append(
        "manifest status is not frozen"
    )

if manifest.get("integrity_gate") != "19/19":
    errors.append(
        "integrity gate is not 19/19"
    )

files = manifest.get(
    "implementation_sha256",
    {},
)

for relative, expected in files.items():
    path = ROOT / relative

    if not path.exists():
        errors.append(
            f"missing: {relative}"
        )
        continue

    actual = sha256(path)

    if actual != expected:
        errors.append(
            f"drift: {relative}"
        )

required = manifest.get(
    "required_invariants",
    [],
)

if len(required) != 19:
    errors.append(
        "required invariant count is not 19"
    )

if len(set(required)) != 19:
    errors.append(
        "required invariants contain duplicates"
    )

if errors:
    print(
        "MODULE 23 PARITY VERIFY: DRIFT"
    )

    for error in errors:
        print("  " + error)

    sys.exit(1)

print("MODULE 23 PARITY VERIFY: PASS")
print(
    "Integrity gate:",
    manifest["integrity_gate"],
)
print(
    "Frozen implementation files:",
    len(files),
)
print(
    "Required invariants:",
    len(required),
)
