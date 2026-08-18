#!/usr/bin/env python3
import pathlib, json, hashlib, subprocess

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

def write_json(path, data):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)

base = pathlib.Path("benchmark/generalization/tasks")
base.mkdir(parents=True, exist_ok=True)

# --- Tanuló taskok ---

# Task 1: ReflectH + Translate(1,0) 3x3
train_1 = {
    "train": [
        {
            "input": [
                [1, 2, 3],
                [4, 5, 6],
                [7, 8, 9]
            ],
            "output": [
                [0, 3, 2],
                [0, 6, 5],
                [0, 9, 8]
            ]
        }
    ],
    "test": [
        {
            "input": [
                [9, 8, 7],
                [6, 5, 4],
                [3, 2, 1]
            ],
            "output": [
                [0, 7, 8],
                [0, 4, 5],
                [0, 1, 2]
            ]
        }
    ]
}

# Task 2: ReflectH + Translate(1,0) 4x4
train_2 = {
    "train": [
        {
            "input": [
                [1, 2, 3, 4],
                [5, 6, 7, 8],
                [9, 1, 2, 3],
                [4, 5, 6, 7]
            ],
            "output": [
                [0, 4, 3, 2],
                [0, 8, 7, 6],
                [0, 3, 2, 1],
                [0, 7, 6, 5]
            ]
        }
    ],
    "test": [
        {
            "input": [
                [7, 6, 5, 4],
                [3, 2, 1, 9],
                [8, 7, 6, 5],
                [4, 3, 2, 1]
            ],
            "output": [
                [0, 4, 5, 6],
                [0, 9, 1, 2],
                [0, 5, 6, 7],
                [0, 1, 2, 3]
            ]
        }
    ]
}

# Task 3: ReflectH + Translate(1,0) 5x5 (más színekkel)
train_3 = {
    "train": [
        {
            "input": [
                [9, 0, 8, 0, 7],
                [0, 6, 0, 5, 0],
                [4, 0, 3, 0, 2],
                [0, 1, 0, 9, 0],
                [8, 0, 7, 0, 6]
            ],
            "output": [
                [0, 7, 0, 8, 0],
                [0, 0, 5, 0, 6],
                [0, 2, 0, 3, 0],
                [0, 0, 9, 0, 1],
                [0, 6, 0, 7, 0]
            ]
        }
    ],
    "test": [
        {
            "input": [
                [0, 8, 0, 7, 0],
                [6, 0, 5, 0, 4],
                [0, 3, 0, 2, 0],
                [1, 0, 9, 0, 8],
                [0, 7, 0, 6, 0]
            ],
            "output": [
                [0, 0, 7, 0, 8],
                [0, 4, 0, 5, 0],
                [0, 0, 2, 0, 3],
                [0, 8, 0, 9, 0],
                [0, 0, 6, 0, 7]
            ]
        }
    ]
}

# --- Held-out task ---
# 6x6, más színek, más pozíciók, de ugyanaz a szabály.
heldout_1 = {
    "train": [],
    "test": [
        {
            "input": [
                [3, 0, 2, 0, 1, 0],
                [0, 4, 0, 5, 0, 6],
                [7, 0, 8, 0, 9, 0],
                [0, 1, 0, 2, 0, 3],
                [4, 0, 5, 0, 6, 0],
                [0, 7, 0, 8, 0, 9]
            ],
            "output": [
                [0, 0, 1, 0, 2, 0],
                [0, 6, 0, 5, 0, 4],
                [0, 0, 9, 0, 8, 0],
                [0, 3, 0, 2, 0, 1],
                [0, 0, 6, 0, 5, 0],
                [0, 9, 0, 8, 0, 7]
            ]
        }
    ]
}

# JSON fájlok írása
write_json(base / "train_task_001.json", train_1)
write_json(base / "train_task_002.json", train_2)
write_json(base / "train_task_003.json", train_3)
write_json(base / "heldout_task_001.json", heldout_1)

# Hash-ek rögzítése
hash_lines = []
for f in sorted(base.glob("*.json")):
    digest = hashlib.sha256(f.read_bytes()).hexdigest()
    hash_lines.append(f"{f.name}  {digest}")

write_file(base / "hashes.txt", "\n".join(hash_lines) + "\n")

print("[1] Benchmark JSON fájlok és hash-ek létrehozva.")
print("Fájlok:")
for f in sorted(base.glob("*.json")):
    print(" -", f)
print("\nHash-ek:")
print((base / "hashes.txt").read_text())
