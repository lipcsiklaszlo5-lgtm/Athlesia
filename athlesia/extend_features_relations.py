#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
FEATURES_DIR = os.path.join(PROJECT, "crates", "athlesia-features")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. lib.rs frissítése: FeatureVector bővítése relációs mezőkkel + Default
lib_path = os.path.join(FEATURES_DIR, "src", "lib.rs")
content = pathlib.Path(lib_path).read_text()

# 1a. Import bővítése
content = content.replace(
    "use athlesia_perception::{segment, touches};",
    "use athlesia_perception::{segment, touches, contains, distance_between, relative_direction};"
)

# 1b. Default derive hozzáadása a FeatureVector-hoz
content = content.replace(
    "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\npub struct FeatureVector {",
    "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]\npub struct FeatureVector {"
)

# 1c. Új mezők hozzáadása a struktúrához
content = content.replace(
    "    pub symmetric_v: bool,\n}",
    "    pub symmetric_v: bool,\n    pub contains_pairs: u8,\n    pub min_distance_category: u8,\n    pub dominant_direction: (i8, i8),\n}"
)

# 1d. extract_features függvény cseréje
old_fn = '''pub fn extract_features(grid: &Grid) -> FeatureVector {
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

    let has_hole = detect_hole(grid);
    let (symmetric_h, symmetric_v) = bounding_box_symmetry(grid);

    FeatureVector {
        object_count,
        color_counts,
        touching_pairs,
        has_hole,
        symmetric_h,
        symmetric_v,
    }
}'''

new_fn = '''pub fn extract_features(grid: &Grid) -> FeatureVector {
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

    // Tartalmazási párok száma
    let mut contains_pairs = 0;
    for i in 0..objects.len() {
        for j in (i + 1)..objects.len() {
            if contains(&objects[i], &objects[j]) || contains(&objects[j], &objects[i]) {
                contains_pairs += 1;
            }
        }
    }

    // Minimális távolság kategória
    let mut min_distance_category = 0u8;
    if objects.len() >= 2 {
        let mut any_touching = false;
        let mut min_dist = f64::MAX;
        for i in 0..objects.len() {
            for j in (i + 1)..objects.len() {
                if touches(&objects[i], &objects[j]) {
                    any_touching = true;
                }
                let d = distance_between(&objects[i], &objects[j]);
                if d < min_dist {
                    min_dist = d;
                }
            }
        }
        if any_touching {
            min_distance_category = 1;
        } else if min_dist <= 2.0 {
            min_distance_category = 2;
        } else {
            min_distance_category = 3;
        }
    }

    // Domináns relatív irány az objektumpárok között
    let mut dir_counts: std::collections::HashMap<(i8, i8), usize> = std::collections::HashMap::new();
    for i in 0..objects.len() {
        for j in (i + 1)..objects.len() {
            let dir = relative_direction(&objects[i], &objects[j]);
            *dir_counts.entry(dir).or_insert(0) += 1;
        }
    }
    let dominant_direction = dir_counts
        .into_iter()
        .max_by_key(|(dir, count)| (*count, std::cmp::Reverse(*dir)))
        .map(|(dir, _)| dir)
        .unwrap_or((0, 0));

    let has_hole = detect_hole(grid);
    let (symmetric_h, symmetric_v) = bounding_box_symmetry(grid);

    FeatureVector {
        object_count,
        color_counts,
        touching_pairs,
        has_hole,
        symmetric_h,
        symmetric_v,
        contains_pairs,
        min_distance_category,
        dominant_direction,
    }
}'''

if old_fn not in content:
    print("[ERROR] Nem találom a régi extract_features függvényt.")
    sys.exit(1)

content = content.replace(old_fn, new_fn)
write_file(lib_path, content)
print("[INFO] FeatureVector bővítve relációs mezőkkel, extract_features frissítve.")

# 2. Metalearner teszt frissítése: használja a Default-ot
ml_test_path = os.path.join(PROJECT, "crates", "athlesia-metalearner", "tests", "context_metalearner_test.rs")
ml_test_content = pathlib.Path(ml_test_path).read_text()

old_fv = '''fn fv(object_count: u8, touching_pairs: u8) -> FeatureVector {
    FeatureVector {
        object_count,
        color_counts: [0; 4],
        touching_pairs,
        has_hole: false,
        symmetric_h: false,
        symmetric_v: false,
    }
}'''

new_fv = '''fn fv(object_count: u8, touching_pairs: u8) -> FeatureVector {
    FeatureVector {
        object_count,
        color_counts: [0; 4],
        touching_pairs,
        ..Default::default()
    }
}'''

if old_fv in ml_test_content:
    ml_test_content = ml_test_content.replace(old_fv, new_fv)
    write_file(ml_test_path, ml_test_content)
    print("[INFO] Metalearner teszt frissítve Default használatával.")
else:
    print("[WARN] Nem találtam az fv függvényt a metalearner tesztben.")

# 3. Új teszt a relációs jellemzőkre
relation_test = r'''
use athlesia_features::extract_features;
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn detects_contains_relation_feature() {
    let grid = build_grid([
        [1, 1, 1, 0, 0],
        [1, 2, 1, 0, 0],
        [1, 1, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let fv = extract_features(&grid);
    assert_eq!(fv.contains_pairs, 1);
    assert_eq!(fv.object_count, 2);
}

#[test]
fn detects_distance_category_touching() {
    let grid = build_grid([
        [1, 2, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let fv = extract_features(&grid);
    assert_eq!(fv.min_distance_category, 1);
}

#[test]
fn detects_dominant_direction() {
    let grid = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 2],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let fv = extract_features(&grid);
    // A 2-es objektum jobbra és lefelé van az 1-eshez képest
    assert_eq!(fv.dominant_direction, (1, 1));
}
'''

write_file(os.path.join(FEATURES_DIR, "tests", "relation_features_test.rs"), relation_test)
print("[INFO] Relációs jellemzők tesztje hozzáadva.")

# 4. Tesztek futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-features"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Features relációs tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Features relációs tesztek zöldek.")

result = subprocess.run(["cargo", "test", "-p", "athlesia-metalearner"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Metalearner tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Metalearner tesztek zöldek.")

# 5. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Extend FeatureVector with relational fields"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
