#!/usr/bin/env python3
import os, subprocess, pathlib

def section(title):
    print("\n" + "=" * 60)
    print(title)
    print("=" * 60)

def run(cmd, timeout=10):
    print("\n$ " + cmd)
    try:
        res = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=timeout)
        print(res.stdout)
        if res.stderr:
            print("STDERR:", res.stderr)
    except subprocess.TimeoutExpired:
        print("TIMEOUT")
    except Exception as e:
        print("ERROR:", e)

# 1. Aktuális könyvtár
section("CURRENT DIRECTORY")
print(os.getcwd())

# 2. Workspace Cargo.toml
section("WORKSPACE Cargo.toml")
p = pathlib.Path("Cargo.toml")
if p.exists():
    print(p.read_text())
else:
    print("Nincs workspace Cargo.toml ebben a könyvtárban.")

# 3. Crate-ek listája (Cargo.toml fájlok, kivéve target/.git)
section("CRATE MANIFEST FILES")
run("find . -name Cargo.toml -not -path './target/*' -not -path './.git/*' | sort")

# 4. Rust forrásfájlok listája (első 150)
section("RUST SOURCE FILES (first 150)")
run("find . -name '*.rs' -not -path './target/*' -not -path './.git/*' | sort | head -150")

# 5. Cargo metadata (csomagok és függőségek, röviden)
section("CARGO METADATA (packages)")
run("cargo metadata --no-deps --format-version 1 | python3 -c \"import sys, json; data=json.load(sys.stdin); [print(p['name'], p['version'], p['manifest_path']) for p in data.get('packages', [])]\"")

# 6. Főbb lib.rs / main.rs fájlok első 30 sora (ha vannak)
section("KEY SOURCE FILES (first 30 lines each)")
for root, dirs, files in os.walk('crates'):
    dirs[:] = [d for d in dirs if d not in {'target'}]
    for f in files:
        if f in ('lib.rs', 'main.rs') and root.endswith('src'):
            path = os.path.join(root, f)
            print(f"\n--- {path} ---")
            with open(path, 'r') as fh:
                lines = fh.readlines()[:30]
                print(''.join(lines))

print("\n[INFO] Inspect script befejezte.")
