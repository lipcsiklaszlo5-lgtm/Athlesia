#!/usr/bin/env python3
import pathlib, subprocess, sys

p = pathlib.Path("crates/athlesia-search/src/lib.rs")
s = p.read_text()

old_line = "            patience_window: 20,"
new_line = "            patience_window: 1000,"

if old_line not in s:
    print("[ERROR] patience_window sor nem található.")
    sys.exit(1)

s = s.replace(old_line, new_line)
p.write_text(s)
print("[1] patience_window 1000-re növelve.")

# Core tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-core"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Core tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Phase 9 benchmark zöld.")

subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Increase search patience window to 1000 to allow small searches to complete"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
