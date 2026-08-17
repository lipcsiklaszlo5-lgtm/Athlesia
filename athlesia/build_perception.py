#!/usr/bin/env python3
import os, subprocess, sys, pathlib

# A jelenlegi könyvtár a projekt gyökere (athlesia)
PROJECT = "."
TYPES_LIB = os.path.join(PROJECT, "crates", "athlesia-types", "src", "lib.rs")
WORKSPACE_TOML = os.path.join(PROJECT, "Cargo.toml")

# Segédfüggvény
def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Coord típus hozzáadása az athlesia-types-hez
types_content = pathlib.Path(TYPES_LIB).read_text()
if "pub struct Coord" not in types_content:
    types_content += """

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coord {
    pub x: i8,
    pub y: i8,
}
"""
    write_file(TYPES_LIB, types_content)
    print("[INFO] Coord típus hozzáadva.")

# 2. Workspace Cargo.toml frissítése
workspace_content = pathlib.Path(WORKSPACE_TOML).read_text()
if "athlesia-perception" not in workspace_content:
    workspace_content = workspace_content.replace(
        'members = ["crates/athlesia-types", "crates/athlesia-executor"]',
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception"]'
    )
    write_file(WORKSPACE_TOML, workspace_content)
    print("[INFO] Workspace frissítve.")

# 3. Perception crate létrehozása
PERCEPTION_DIR = os.path.join(PROJECT, "crates", "athlesia-perception")
os.makedirs(os.path.join(PERCEPTION_DIR, "src"), exist_ok=True)
os.makedirs(os.path.join(PERCEPTION_DIR, "tests"), exist_ok=True)

write_file(os.path.join(PERCEPTION_DIR, "Cargo.toml"), '''[package]
name = "athlesia-perception"
version = "0.1.0"
edition = "2021"

[dependencies]
athlesia-types = { path = "../athlesia-types" }
''')

write_file(os.path.join(PERCEPTION_DIR, "src", "lib.rs"), r'''
use athlesia_types::{Color, Coord, Grid, GRID_SIZE};

#[derive(Debug, Clone)]
pub struct GameObject {
    pub id: u64,
    pub color: Color,
    pub cells: Vec<Coord>,
}

/// Flood-fill alapú összefüggő komponens keresés.
/// 0 szín = háttér (nem objektum).
pub fn segment(grid: &Grid) -> Vec<GameObject> {
    let mut visited = [[false; GRID_SIZE]; GRID_SIZE];
    let mut objects = Vec::new();
    let mut next_id = 0u64;

    for i in 0..GRID_SIZE {
        for j in 0..GRID_SIZE {
            if visited[i][j] {
                continue;
            }
            let color = grid.cells[i][j];
            if color == 0 {
                visited[i][j] = true;
                continue;
            }

            let mut stack = vec![(i as i8, j as i8)];
            let mut cells = Vec::new();
            visited[i][j] = true;

            while let Some((x, y)) = stack.pop() {
                cells.push(Coord { x, y });

                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = x + dx;
                    let ny = y + dy;
                    if nx >= 0 && nx < GRID_SIZE as i8 && ny >= 0 && ny < GRID_SIZE as i8 {
                        let ni = nx as usize;
                        let nj = ny as usize;
                        if !visited[ni][nj] && grid.cells[ni][nj] == color {
                            visited[ni][nj] = true;
                            stack.push((nx, ny));
                        }
                    }
                }
            }

            objects.push(GameObject {
                id: next_id,
                color,
                cells,
            });
            next_id += 1;
        }
    }

    objects
}

/// Két objektum akkor érintkezik, ha van olyan cellájuk,
/// amelyek egymás mellett vannak (Manhattan-távolság = 1).
pub fn touches(a: &GameObject, b: &GameObject) -> bool {
    for ca in &a.cells {
        for cb in &b.cells {
            if (ca.x - cb.x).abs() + (ca.y - cb.y).abs() == 1 {
                return true;
            }
        }
    }
    false
}
''')

# 4. Perception tesztek
write_file(os.path.join(PERCEPTION_DIR, "tests", "segmentation_test.rs"), r'''
use athlesia_perception::{segment, touches};
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn segment_two_separate_objects() {
    let grid = build_grid([
        [1, 1, 0, 0, 0],
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 2, 2],
        [0, 0, 0, 2, 0],
    ]);

    let objects = segment(&grid);
    assert_eq!(objects.len(), 2);
    assert_eq!(objects[0].color, 1);
    assert_eq!(objects[1].color, 2);
}

#[test]
fn segment_ignores_background() {
    let grid = build_grid([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let objects = segment(&grid);
    assert_eq!(objects.len(), 1);
    assert_eq!(objects[0].cells.len(), 1);
}

#[test]
fn touches_false_for_diagonal() {
    let grid = build_grid([
        [1, 0, 0, 0, 0],
        [0, 2, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let objects = segment(&grid);
    assert_eq!(objects.len(), 2);
    assert!(!touches(&objects[0], &objects[1]));
}

#[test]
fn touches_true_for_side_by_side() {
    let grid = build_grid([
        [1, 2, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let objects = segment(&grid);
    assert_eq!(objects.len(), 2);
    assert!(touches(&objects[0], &objects[1]));
}
''')

print("[INFO] Perception crate létrehozva.")

# 5. Cargo test
result = subprocess.run(["cargo", "test", "-p", "athlesia-perception"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Perception tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Perception tesztek zöldek.")

# 6. Git commit / push (ha van git)
try:
    subprocess.run(["git", "rev-parse", "--is-inside-work-tree"], check=True, capture_output=True)
    subprocess.run(["git", "add", "-A"], check=True)
    subprocess.run(["git", "commit", "-m", "Add athlesia-perception module with segmentation and touches"], check=True)
    subprocess.run(["git", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre (valószínűleg nincs git repó).")
