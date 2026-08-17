#!/usr/bin/env python3
import os, subprocess, sys, pathlib

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Lib.rs kiegészítése FrameDelta, ObjectGraph, fingerprint, tracking
p = pathlib.Path("crates/athlesia-perception/src/lib.rs")
s = p.read_text()

# Új importok
if "use std::collections::HashSet;" not in s:
    s = s.replace(
        "use std::collections::HashMap;",
        "use std::collections::{HashMap, HashSet};"
    )

# Új struktúrák és függvények a fájl végéhez
additions = r'''

/// Az előző és aktuális képkocka közötti változások.
#[derive(Debug, Clone, Default)]
pub struct FrameDelta {
    pub added: Vec<Coord>,
    pub removed: Vec<Coord>,
    pub changed: Vec<Coord>,
}

/// Két grid közötti cellánkénti különbség.
pub fn diff_grids(prev: &Grid, current: &Grid) -> FrameDelta {
    let mut delta = FrameDelta::default();
    if prev.width != current.width || prev.height != current.height {
        // Méretváltozás esetén egyszerűen minden nem-nulla cella added, és minden prev cella removed
        for y in 0..current.height as i8 {
            for x in 0..current.width as i8 {
                if current.get(x, y).map_or(false, |c| c != Color(0)) {
                    delta.added.push(Coord { x, y });
                }
            }
        }
        for y in 0..prev.height as i8 {
            for x in 0..prev.width as i8 {
                if prev.get(x, y).map_or(false, |c| c != Color(0)) {
                    delta.removed.push(Coord { x, y });
                }
            }
        }
        return delta;
    }

    for y in 0..current.height as i8 {
        for x in 0..current.width as i8 {
            let old = prev.get(x, y);
            let new = current.get(x, y);
            match (old, new) {
                (Some(o), Some(n)) if o != n => delta.changed.push(Coord { x, y }),
                (None, Some(_)) => delta.added.push(Coord { x, y }),
                (Some(_), None) => delta.removed.push(Coord { x, y }),
                _ => {}
            }
        }
    }
    delta
}

/// Objektumgráf: az objektumok listája, és a köztük lévő relációk.
#[derive(Debug, Clone, Default)]
pub struct ObjectGraph {
    pub objects: Vec<GameObject>,
    pub touching_pairs: Vec<(usize, usize)>,
    pub contains_pairs: Vec<(usize, usize)>,
}

/// A percepciós csővezeték kimenete: az objektumgráf és a frame-delta.
#[derive(Debug, Clone, Default)]
pub struct PerceptionOutput {
    pub graph: ObjectGraph,
    pub delta: FrameDelta,
}

/// Objektum-ujjlenyomat: forgatás- és tükrözés-invariáns leírás.
pub fn shape_fingerprint(obj: &GameObject) -> (Color, Vec<(i8, i8)>) {
    let (min_x, min_y, _, _) = bounding_box(obj);
    let mut rel: Vec<(i8, i8)> = obj.cells.iter().map(|c| (c.x - min_x, c.y - min_y)).collect();
    rel.sort_unstable();
    (obj.color, rel)
}

/// Két frame objektumainak párosítása ujjlenyomat alapján.
/// Visszaadja a matched párokat (prev_index, current_index).
pub fn track_objects(prev_objects: &[GameObject], current_objects: &[GameObject]) -> Vec<(usize, usize)> {
    let mut matches = Vec::new();
    let mut used_current = HashSet::new();

    for (pi, p_obj) in prev_objects.iter().enumerate() {
        let fp_p = shape_fingerprint(p_obj);
        for (ci, c_obj) in current_objects.iter().enumerate() {
            if used_current.contains(&ci) {
                continue;
            }
            if shape_fingerprint(c_obj) == fp_p {
                matches.push((pi, ci));
                used_current.insert(ci);
                break;
            }
        }
    }
    matches
}

/// A jelenlegi gridből teljes PerceptionOutput-ot készít.
pub fn perceive(prev: Option<&Grid>, current: &Grid) -> PerceptionOutput {
    let objects = segment(current);
    let mut graph = ObjectGraph {
        objects,
        touching_pairs: Vec::new(),
        contains_pairs: Vec::new(),
    };

    // Relációk kiszámítása
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
}
'''
s += additions
write_file(p, s)
print("[1] Perception lib.rs kiegészítve.")

# 2. Tesztek hozzáadása
test_content = r'''
use athlesia_perception::{
    segment, diff_grids, shape_fingerprint, track_objects, perceive
};
use athlesia_types::{Grid, Color};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn diff_grids_detects_change() {
    let prev = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let cur = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let delta = diff_grids(&prev, &cur);
    assert_eq!(delta.changed.len(), 2); // a (0,0) és (0,1) is változott
}

#[test]
fn shape_fingerprint_is_translation_invariant() {
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
    assert_eq!(objs1.len(), 1);
    assert_eq!(objs2.len(), 1);

    let fp1 = shape_fingerprint(&objs1[0]);
    let fp2 = shape_fingerprint(&objs2[0]);
    assert_eq!(fp1, fp2);
}

#[test]
fn track_objects_matches_by_fingerprint() {
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

    let prev_objs = segment(&g1);
    let cur_objs = segment(&g2);
    let matches = track_objects(&prev_objs, &cur_objs);
    assert_eq!(matches, vec![(0, 0)]);
}

#[test]
fn perceive_builds_graph_and_delta() {
    let prev = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let cur = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let output = perceive(Some(&prev), &cur);
    assert_eq!(output.graph.objects.len(), 1);
    assert_eq!(output.delta.changed.len(), 2);
}
'''
p = pathlib.Path("crates/athlesia-perception/tests/perception_full_test.rs")
write_file(p, test_content)
print("[2] Perception tesztek hozzáadva.")

# 3. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-perception", "--test", "perception_full_test"], capture_output=True, text=True, check=False)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Perception teljes teszt nem ment át.")
    sys.exit(1)
print("\n[SUCCESS] Perception teljes teszt zöld.")

# 4. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Finalize Perception module with FrameDelta, ObjectGraph, fingerprint, and tracking"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
