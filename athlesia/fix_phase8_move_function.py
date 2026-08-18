#!/usr/bin/env python3
import pathlib, subprocess, sys

p = pathlib.Path("crates/athlesia-search/src/lib.rs")
s = p.read_text()

start = s.find("pub fn search_with_budget")
if start == -1:
    print("[ERROR] search_with_budget kezdet nem található.")
    sys.exit(1)

# Keresd meg a blokk végét: a "\n    None\n}\n" mintát a start után
pattern = "\n    None\n}\n"
end_rel = s.find(pattern, start)
if end_rel == -1:
    print("[ERROR] A blokk vége nem található.")
    sys.exit(1)
end = end_rel + len(pattern)

block = s[start:end]

# Távolítsuk el a blokkot a jelenlegi helyéről
s = s[:start] + s[end:]

# Illesszük a fájl legvégére (a score_grid után)
s = s.rstrip() + "\n\n" + block.rstrip() + "\n"

p.write_text(s)
print("[1] search_with_budget kiemelve és a fájl végére helyezve.")

# Most már a search crate tesztnek át kell mennie.
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-search", "--test", "budget_abort_test"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Search budget teszt nem ment át.")
    sys.exit(1)

# Ha ez átment, futtassuk a teljes search crate-et
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-search"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Search crate tesztek nem mentek át.")
    sys.exit(1)

print("\n[SUCCESS] Search crate tesztek zöldek.")
