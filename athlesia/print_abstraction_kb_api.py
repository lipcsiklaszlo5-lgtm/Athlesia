#!/usr/bin/env python3
import pathlib

def print_file(path):
    p = pathlib.Path(path)
    if not p.exists():
        print(f"[WARNING] {path} nem létezik.")
        return
    print("\n" + "=" * 60)
    print(f"FILE: {path}")
    print("=" * 60)
    print(p.read_text())

print_file("crates/athlesia-abstraction/src/lib.rs")
print_file("crates/athlesia-knowledge/src/lib.rs")
