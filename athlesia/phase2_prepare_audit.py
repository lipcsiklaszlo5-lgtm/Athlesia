#!/usr/bin/env python3
import subprocess, pathlib

def section(title):
    print("\n" + "=" * 60)
    print(title)
    print("=" * 60)

# 1. Teljes workspace teszt futtatása
section("FULL WORKSPACE TEST")
result = subprocess.run(
    ["cargo", "test", "--workspace"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[WARNING] Valamelyik crate tesztje nem ment át. Nézd meg a fenti hibát.")
else:
    print("\n[SUCCESS] Minden teszt zöld az egész workspace-ben.")

# 2. athlesia-structure crate forrásának listája és API kivonat
section("STRUCTURE CRATE FILES")
p = pathlib.Path("crates/athlesia-structure/src")
if p.exists():
    for f in p.glob("*.rs"):
        print("\n---", f, "---")
        print(f.read_text()[:2000])  # első 2000 karakter
else:
    print("Nincs structure crate src könyvtár.")

# 3. Megnézzük a structure crate tesztjeit is, hogy lássuk, mit tud már
section("STRUCTURE CRATE TESTS")
tests_dir = pathlib.Path("crates/athlesia-structure/tests")
if tests_dir.exists():
    for f in tests_dir.glob("*.rs"):
        print("\n---", f, "---")
        print(f.read_text()[:2000])
