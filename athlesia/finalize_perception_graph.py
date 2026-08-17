#!/usr/bin/env python3
import os, subprocess, sys, pathlib

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. ObjectGraph kiegészítése, hash és szimmetria reláció
p = pathlib.Path("crates/athlesia-perception/src/lib.rs")
s = p.read_text()

# Az ObjectGraph struktúra cseréje a hiányzó mezőkkel
old_struct = '''#[derive(Debug, Clone, Default)]
pub struct ObjectGraph {
    pub objects: Vec<GameObject>,
    pub touching_pairs: Vec<(usize, usize)>,
    pub contains_pairs: Vec<(usize, usize)>,
}'''
new_struct = '''#[derive(Debug, Clone, Default)]
pub struct ObjectGraph {
    pub objects: Vec<GameObject>,
    pub touching_pairs: Vec<(usize, usize)>,
    pub contains_pairs: Vec<(usize, usize)>,
    pub same_color_pairs: Vec<(usize, usize)>,
    pub symmetry_pairs: Vec<(usize, usize)>,
}'''
if old_struct in s:
    s = s.replace(old_struct, new_struct)
else:
    print("[ERROR] ObjectGraph struct nem található")
    sys.exit(1)

# shape_fingerprint_hash és szimmetria-ellenőrzés beszúrása a shape_fingerprint után
old_fingerprint = '''pub fn shape_fingerprint(obj: &GameObject) -> (Color, Vec<(i8, i8)>) {
    let (min_x, min_y, _, _) = bounding_box(obj);
    let mut rel: Vec<(i8, i8)> = obj.cells.iter().map(|c| (c.x - min_x, c.y - min_y)).collect();
    rel.sort_unstable();
    (obj.color, rel)
}'''
new_fingerprint = old_fingerprint + r'''

/// Determinisztikus ujjlenyomat hash (FNV-1a).
pub fn shape_fingerprint_hash(obj: &GameObject) -> u64 {
    let (color, rel) = shape_fingerprint(obj);
    let mut hash: u64 = 0xcbf29ce484222325;
    let prime: u64 = 0x100000001b3;

    // Szín belekeverése
    hash ^= color.0 as u64;
    hash = hash.wrapping_mul(prime);

    // Relatív koordináták belekeverése
    for (dx, dy) in rel {
        hash ^= dx as u64;
        hash = hash.wrapping_mul(prime);
        hash ^= dy as u64;
        hash = hash.wrapping_mul(prime);
    }

    hash
}

/// Két objektum szimmetria-relációja a rács középvonalaira tükrözve.
/// Igaz, ha az alakjuk megegyezik (ujjlenyomat hash egyezik),
/// és centroidjaik szimmetrikusak a rács függőleges vagy vízszintes középtengelyére.
fn is_symmetric_pair(a: &GameObject, b: &GameObject, grid: &Grid) -> bool {
    if shape_fingerprint_hash(a) != shape_fingerprint_hash(b) {
        return false;
    }

    let (ax, ay) = centroid(a);
    let (bx, by) = centroid(b);

    let cx = (grid.width as f64 - 1.0) / 2.0;
    let cy = (grid.height as f64 - 1.0) / 2.0;

    // Függőleges tükrözés: b -> (2*cx - bx, by)
    let vx = 2.0 * cx - bx;
    let vy = by;
    if (ax - vx).abs() < 0.1 && (ay - vy).abs() < 0.1 {
        return true;
    }

    // Vízszintes tükrözés: b -> (bx, 2*cy - by)
    let hx = bx;
    let hy = 2.0 * cy - by;
    if (ax - hx).abs() < 0.1 && (ay - hy).abs() < 0.1 {
        return true;
    }

    false
}
'''
if old_fingerprint in s:
    s = s.replace(old_fingerprint, new_fingerprint)
else:
    print("[ERROR] shape_fingerprint blokk nem található")
    sys.exit(1)

# perceive() kiegészítése az új relációkkal
old_perceive = '''    // Relációk kiszámítása
    for i in 0..graph.objects.len() {
        for j in (i + 1)..graph.objects.len() {
            if touches(&graph.objects[i], &graph.objects[j]) {
                graph.touching_pairs.push((i, j));
            }
            if contains(&graph.objects[i], &graph.objects[j]) || contains(&graph.objects[j], &graph.objects[i]) {
                graph.contains_pairs.push((i, j));
            }
        }
    }

    let delta = match prev {
        Some(p) => diff_grids(p, current),
        None => FrameDelta::default(),
    };

    PerceptionOutput { graph, delta }
}'''
new_perceive = '''    // Relációk kiszámítása
    for i in 0..graph.objects.len() {
        for j in (i + 1)..graph.objects.len() {
            if touches(&graph.objects[i], &graph.objects[j]) {
                graph.touching_pairs.push((i, j));
            }
            if contains(&graph.objects[i], &graph.objects[j]) || contains(&graph.objects[j], &graph.objects[i]) {
                graph.contains_pairs.push((i, j));
            }
            if graph.objects[i].color == graph.objects[j].color {
                graph.same_color_pairs.push((i, j));
            }
            if is_symmetric_pair(&graph.objects[i], &graph.objects[j], current) {
                graph.symmetry_pairs.push((i, j));
            }
        }
    }

    let delta = match prev {
        Some(p) => diff_grids(p, current),
        None => FrameDelta::default(),
    };

    PerceptionOutput { graph, delta }
}'''
if old_perceive in s:
    s = s.replace(old_perceive, new_perceive)
else:
    print("[ERROR] perceive reláció blokk nem található")
    sys.exit(1)

write_file(p, s)
print("[1] Perception lib.rs kiegészítve.")

# 2. Tesztek hozzáadása
write_file("crates/athlesia-perception/tests/graph_relations_test.rs", r'''
use athlesia_perception::{perceive, segment, shape_fingerprint_hash};
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn same_color_pairs_are_detected() {
    let grid = build_grid([
        [1, 0, 0, 0, 1],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let output = perceive(None, &grid);
    assert_eq!(output.graph.same_color_pairs, vec![(0, 1)]);
}

#[test]
fn symmetry_pairs_are_detected() {
    let grid = build_grid([
        [1, 0, 0, 0, 1],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let output = perceive(None, &grid);
    // Két egymással szimmetrikus (vízszintesen) objektum
    assert_eq!(output.graph.symmetry_pairs, vec![(0, 1)]);
}

#[test]
fn shape_fingerprint_hash_is_translation_invariant() {
    let g1 = build_grid([
        [1, 1, 0, 0, 0],
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let g2 = build_grid([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 1, 1, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let objs1 = segment(&g1);
    let objs2 = segment(&g2);
    assert_eq!(shape_fingerprint_hash(&objs1[0]), shape_fingerprint_hash(&objs2[0]));
}
''')
print("[2] Perception gráf reláció tesztek hozzáadva.")

# 3. Tesztek futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-perception", "--test", "graph_relations_test"], capture_output=True, text=True, check=False)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Perception gráf reláció tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Perception gráf reláció tesztek zöldek.")

# 4. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Extend Perception ObjectGraph with same-color and symmetry relations"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
