#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

p = pathlib.Path("crates/athlesia-abstraction/Cargo.toml")
s = p.read_text()

# Kicseréljük a teljes tartalmat a helyes, egyszeri deps szekcióval
fixed = '''[package]
name = "athlesia-abstraction"
version = "0.1.0"
edition = "2021"

[dependencies]
athlesia-types = { path = "../athlesia-types" }
athlesia-knowledge = { path = "../athlesia-knowledge" }
athlesia-world-model = { path = "../athlesia-world-model" }
athlesia-hypothesis = { path = "../athlesia-hypothesis" }
'''
p.write_text(fixed)
print("[1] Cargo.toml duplikátum javítva.")

# Abstraction tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-abstraction"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Abstraction tesztek nem mentek át.")
    sys.exit(1)

# Teljes workspace teszt
result = subprocess.run(
    ["cargo", "test", "--workspace", "--no-fail-fast"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Teljes workspace tesztek nem mentek át.")
    sys.exit(1)

print("\n[SUCCESS] Minden teszt zöld.")

subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Fix duplicate dependencies in athlesia-abstraction Cargo.toml"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
