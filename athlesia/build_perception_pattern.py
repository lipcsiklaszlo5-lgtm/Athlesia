#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
PERCEPTION_DIR = os.path.join(PROJECT, "crates", "athlesia-perception")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. pattern.rs létrehozása
pattern_rs = r'''
use crate::Grid;

/// Egy adott (px, py) periódus mennyire illik a rácsra.
/// Azt vizsgálja, hogy a rácson belül minden (r,c) és (r+py, c+px) cella
/// színe milyen arányban egyezik meg.
pub fn periodicity_score(grid: &Grid, period: (usize, usize)) -> f32 {
    let (px, py) = period;
    if px == 0 || py == 0 {
        return 1.0; // nincs eltolás, mindig egyezik
    }
    if px >= grid.width as usize || py >= grid.height as usize {
        return 0.0;
    }

    let mut total = 0usize;
    let mut matching = 0usize;

    for y in 0..(grid.height as usize - py) {
        for x in 0..(grid.width as usize - px) {
            let idx1 = y * grid.width as usize + x;
            let idx2 = (y + py) * grid.width as usize + (x + px);
            total += 1;
            if grid.cells[idx1] == grid.cells[idx2] {
                matching += 1;
            }
        }
    }

    if total == 0 {
        1.0
    } else {
        matching as f32 / total as f32
    }
}

/// A legkisebb periódus, amely a megadott küszöb feletti pontszámot ad.
/// Brute-force keresés az összes (px, py) kombinációra,
/// a legkisebb px*py szorzatú találatot visszaadva.
pub fn detect_periodicity(grid: &Grid, min_score: f32) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize)> = None;
    let mut best_area = usize::MAX;

    for py in 1..=grid.height as usize / 2 {
        for px in 1..=grid.width as usize / 2 {
            let score = periodicity_score(grid, (px, py));
            if score >= min_score {
                let area = px * py;
                if area < best_area {
                    best_area = area;
                    best = Some((px, py));
                }
            }
        }
    }

    best
}
'''
write_file(os.path.join(PERCEPTION_DIR, "src", "pattern.rs"), pattern_rs)
print("[INFO] pattern.rs létrehozva.")

# 2. lib.rs modul regisztrálása
lib_path = os.path.join(PERCEPTION_DIR, "src", "lib.rs")
lib_content = pathlib.Path(lib_path).read_text()
if "pub mod pattern;" not in lib_content:
    lib_content = lib_content.replace(
        "pub mod texture;",
        "pub mod texture;\npub mod pattern;"
    )
    pathlib.Path(lib_path).write_text(lib_content)
    print("[INFO] pattern modul hozzáadva.")
else:
    print("[INFO] pattern modul már létezik.")

# 3. Tesztek
test_content = r'''
use athlesia_perception::pattern::{periodicity_score, detect_periodicity};
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn detects_periodic_pattern() {
    // Sakktábla-szerű 2x2 minta
    let grid = build_grid([
        [1, 2, 1, 2, 1],
        [2, 1, 2, 1, 2],
        [1, 2, 1, 2, 1],
        [2, 1, 2, 1, 2],
        [1, 2, 1, 2, 1],
    ]);
    let period = detect_periodicity(&grid, 1.0);
    assert!(period.is_some());
    let (px, py) = period.unwrap();
    // A legkisebb periódus 2x2, mert az 1x2 vagy 2x1 nem ad tökéletes egyezést
    assert_eq!((px, py), (2, 2));
}

#[test]
fn periodicity_score_is_one_for_exact_repeat() {
    let grid = build_grid([
        [1, 1, 2, 2, 1],
        [1, 1, 2, 2, 1],
        [2, 2, 1, 1, 2],
        [2, 2, 1, 1, 2],
        [1, 1, 2, 2, 1],
    ]);
    // 2x2 periódus tökéletesen illeszkedik
    let score = periodicity_score(&grid, (2, 2));
    assert_eq!(score, 1.0);
}

#[test]
fn no_periodicity_on_random_grid() {
    let grid = build_grid([
        [1, 2, 3, 4, 5],
        [5, 4, 3, 2, 1],
        [1, 3, 2, 4, 5],
        [5, 2, 4, 3, 1],
        [2, 4, 1, 5, 3],
    ]);
    // Itt 4-es, 5-ös színek is vannak, de a rács 5x5
    let period = detect_periodicity(&grid, 0.99);
    assert!(period.is_none());
}
'''
write_file(os.path.join(PERCEPTION_DIR, "tests", "pattern_test.rs"), test_content)
print("[INFO] pattern_test.rs létrehozva.")

# 4. Tesztek futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-perception", "--test", "pattern_test"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Perception pattern tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Perception pattern tesztek zöldek.")

# 5. Git commit és push
try:
    subprocess.run(["git", "add", "-A"], check=True)
    subprocess.run(["git", "commit", "-m", "Add pattern periodicity detection to perception"], check=True)
    subprocess.run(["git", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
