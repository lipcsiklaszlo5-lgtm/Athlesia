#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
FEATURES_DIR = os.path.join(PROJECT, "crates", "athlesia-features")
WORKSPACE_TOML = os.path.join(PROJECT, "Cargo.toml")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Workspace frissítése
ws = pathlib.Path(WORKSPACE_TOML).read_text()
if "athlesia-features" not in ws:
    ws = ws.replace(
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model"]',
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features"]'
    )
    write_file(WORKSPACE_TOML, ws)
    print("[INFO] Workspace frissítve.")

# 2. Features crate létrehozása
os.makedirs(os.path.join(FEATURES_DIR, "src"), exist_ok=True)
os.makedirs(os.path.join(FEATURES_DIR, "tests"), exist_ok=True)

write_file(os.path.join(FEATURES_DIR, "Cargo.toml"), '''[package]
name = "athlesia-features"
version = "0.1.0"
edition = "2021"

[dependencies]
athlesia-types = { path = "../athlesia-types" }
athlesia-perception = { path = "../athlesia-perception" }
''')

write_file(os.path.join(FEATURES_DIR, "src", "lib.rs"), r'''
use athlesia_types::{Grid, GRID_SIZE};
use athlesia_perception::{segment, touches};

/// FeatureVector: ezeket a jellemzőket fogja később a MetaLearner használni.
/// A Hash miatt könnyen lehet HashMap kulcs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeatureVector {
    pub object_count: u8,
    pub color_counts: [u8; 4],
    pub touching_pairs: u8,
}

pub fn extract_features(grid: &Grid) -> FeatureVector {
    let objects = segment(grid);
    let object_count = objects.len() as u8;

    // Szín gyakoriságok
    let mut color_counts = [0u8; 4];
    for row in &grid.cells {
        for &cell in row {
            if cell < 4 {
                color_counts[cell as usize] += 1;
            }
        }
    }

    // Érintkező párok száma
    let mut touching_pairs = 0;
    for i in 0..objects.len() {
        for j in (i + 1)..objects.len() {
            if touches(&objects[i], &objects[j]) {
                touching_pairs += 1;
            }
        }
    }

    FeatureVector {
        object_count,
        color_counts,
        touching_pairs,
    }
}
''')

# 3. Teszt
write_file(os.path.join(FEATURES_DIR, "tests", "features_test.rs"), r'''
use athlesia_features::extract_features;
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn extract_basic_features() {
    let grid = build_grid([
        [1, 1, 0, 0, 0],
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 2, 2],
        [0, 0, 0, 2, 0],
    ]);

    let fv = extract_features(&grid);

    assert_eq!(fv.object_count, 2);
    assert_eq!(fv.color_counts[1], 3);
    assert_eq!(fv.color_counts[2], 3);
    assert_eq!(fv.touching_pairs, 0);
}

#[test]
fn detect_touching_pair() {
    let grid = build_grid([
        [1, 2, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let fv = extract_features(&grid);

    assert_eq!(fv.object_count, 2);
    assert_eq!(fv.touching_pairs, 1);
}
''')

print("[INFO] Features crate létrehozva.")

# 4. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-features"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Features tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Features tesztek zöldek.")

# 5. Git commit és push
try:
    subprocess.run(["git", "add", "-A"], check=True)
    subprocess.run(["git", "commit", "-m", "Add athlesia-features module"], check=True)
    subprocess.run(["git", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
