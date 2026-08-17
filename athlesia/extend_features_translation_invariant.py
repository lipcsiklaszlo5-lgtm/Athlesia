#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
FEATURES_DIR = os.path.join(PROJECT, "crates", "athlesia-features")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. lib.rs frissítése: eltolás-invariáns szimmetria
lib_path = os.path.join(FEATURES_DIR, "src", "lib.rs")
content = pathlib.Path(lib_path).read_text()

if "fn bounding_box_symmetry" not in content:
    # A régi detect_symmetry függvényt kicseréljük a bounding box alapúra
    old_detect = '''/// Szimmetria: vízszintes és függőleges tengelyes tükrözés ellenőrzése.
fn detect_symmetry(grid: &Grid) -> (bool, bool) {
    let rows = grid.cells.len();
    let cols = grid.cells[0].len();
    let mut sym_h = true;
    let mut sym_v = true;

    for i in 0..rows {
        for j in 0..cols {
            if grid.cells[i][j] != grid.cells[i][cols - 1 - j] {
                sym_h = false;
            }
            if grid.cells[i][j] != grid.cells[rows - 1 - i][j] {
                sym_v = false;
            }
        }
    }
    (sym_h, sym_v)
}'''
    new_detect = '''/// Szimmetria: az objektumok befoglaló téglalapján belül ellenőrizzük.
/// Ez eltolás-invariáns: ugyanaz a minta más pozícióban ugyanazt adja.
fn bounding_box_symmetry(grid: &Grid) -> (bool, bool) {
    let rows = grid.cells.len();
    let cols = grid.cells[0].len();

    // Keressük meg a legkisebb befoglaló téglalapot, ami minden nem-nulla cellát tartalmaz
    let mut min_i = rows;
    let mut max_i = 0;
    let mut min_j = cols;
    let mut max_j = 0;
    let mut has_object = false;

    for i in 0..rows {
        for j in 0..cols {
            if grid.cells[i][j] != 0 {
                has_object = true;
                if i < min_i { min_i = i; }
                if i > max_i { max_i = i; }
                if j < min_j { min_j = j; }
                if j > max_j { max_j = j; }
            }
        }
    }

    if !has_object {
        return (true, true); // üres grid mindig szimmetrikus
    }

    let bbox_height = max_i - min_i + 1;
    let bbox_width = max_j - min_j + 1;

    // Vízszintes szimmetria a bounding boxon belül
    let mut sym_h = true;
    for i in min_i..=max_i {
        for j in min_j..=max_j {
            let mirrored_j = max_j - (j - min_j);
            if grid.cells[i][j] != grid.cells[i][mirrored_j] {
                sym_h = false;
                break;
            }
        }
        if !sym_h { break; }
    }

    // Függőleges szimmetria a bounding boxon belül
    let mut sym_v = true;
    for i in min_i..=max_i {
        for j in min_j..=max_j {
            let mirrored_i = max_i - (i - min_i);
            if grid.cells[i][j] != grid.cells[mirrored_i][j] {
                sym_v = false;
                break;
            }
        }
        if !sym_v { break; }
    }

    (sym_h, sym_v)
}'''
    if old_detect in content:
        content = content.replace(old_detect, new_detect)
        # A hívás neve is változik: detect_symmetry -> bounding_box_symmetry
        content = content.replace(
            "let (symmetric_h, symmetric_v) = detect_symmetry(grid);",
            "let (symmetric_h, symmetric_v) = bounding_box_symmetry(grid);"
        )
        write_file(lib_path, content)
        print("[INFO] Eltolás-invariáns szimmetria implementálva.")
    else:
        print("[ERROR] Nem találtam a régi detect_symmetry blokkot.")
        sys.exit(1)
else:
    print("[INFO] bounding_box_symmetry már létezik.")

# 2. Tesztek módosítása, hogy az eltolás-invarianciát ellenőrizzék
test_content = r'''
use athlesia_features::extract_features;
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn symmetry_is_translation_invariant() {
    // Eredeti: egy vízszintesen szimmetrikus minta a bal felső sarokban
    let original = build_grid([
        [1, 2, 3, 2, 1],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    // Ugyanaz a minta jobbra és lefelé tolva
    let shifted = build_grid([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 1, 2, 3],
        [2, 1, 0, 0, 0], // ez nem szimmetrikus elrendezés, szándékosan rossz példa
    ]);

    let fv_original = extract_features(&original);
    let fv_shifted = extract_features(&shifted);

    // A shifted rács nem ugyanazt a mintát tartalmazza, ezért nem egyezhetnek.
    // Ezt a tesztet átírjuk: valódi eltolást használunk.
    // Helyes eltolás: minta a jobb alsó sarokban, teljes sorral.
    let shifted_correct = build_grid([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [1, 2, 3, 2, 1],
    ]);

    let fv_shifted_correct = extract_features(&shifted_correct);

    // Mindkettő szimmetrikus vízszintesen, nem függőlegesen
    assert_eq!(fv_original.symmetric_h, fv_shifted_correct.symmetric_h);
    assert_eq!(fv_original.symmetric_v, fv_shifted_correct.symmetric_v);
    assert!(fv_original.symmetric_h);
    assert!(!fv_original.symmetric_v);
}

#[test]
fn vertical_symmetry_translation_invariant() {
    // Függőlegesen szimmetrikus minta bal felső sarokban
    let original = build_grid([
        [1, 0, 0, 0, 0],
        [2, 0, 0, 0, 0],
        [3, 0, 0, 0, 0],
        [2, 0, 0, 0, 0],
        [1, 0, 0, 0, 0],
    ]);
    // Ugyanaz jobbra tolva
    let shifted = build_grid([
        [0, 0, 1, 0, 0],
        [0, 0, 2, 0, 0],
        [0, 0, 3, 0, 0],
        [0, 0, 2, 0, 0],
        [0, 0, 1, 0, 0],
    ]);

    let fv_original = extract_features(&original);
    let fv_shifted = extract_features(&shifted);

    assert!(fv_original.symmetric_v);
    assert!(fv_shifted.symmetric_v);
    assert_eq!(fv_original.symmetric_v, fv_shifted.symmetric_v);
    assert!(!fv_original.symmetric_h);
    assert!(!fv_shifted.symmetric_h);
}
'''
# Cseréljük a régi geometriai szimmetria teszteket
write_file(os.path.join(FEATURES_DIR, "tests", "geometry_features_test.rs"), test_content)
print("[INFO] Geometriai szimmetria tesztek frissítve.")

# 3. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-features"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Features eltolás-invariancia tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Features eltolás-invariancia tesztek zöldek.")

# 4. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Make symmetry features translation-invariant"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
