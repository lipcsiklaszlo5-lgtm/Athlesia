#!/usr/bin/env python3
import pathlib

def print_file(path, max_chars=8000):
    p = pathlib.Path(path)
    if not p.exists():
        print(f"[WARNING] {path} nem létezik.")
        return
    print("\n" + "=" * 60)
    print(f"FILE: {path}")
    print("=" * 60)
    content = p.read_text()
    print(content[:max_chars])
    if len(content) > max_chars:
        print(f"\n... (truncated, total {len(content)} chars)")

# 1. structure teljes forrása
print_file("crates/athlesia-structure/src/lib.rs")

# 2. synthesis teljes forrása
print_file("crates/athlesia-synthesis/src/lib.rs")

# 3. types: PrimName, Params, Program definíciók
print_file("crates/athlesia-types/src/lib.rs", max_chars=6000)

# 4. executor: BlockMap és egyéb primitívek (első 6000 karakter)
print_file("crates/athlesia-executor/src/lib.rs", max_chars=6000)

print("\n[INFO] API print kész.")
