#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
STRUCT_DIR = os.path.join(PROJECT, "crates", "athlesia-structure")
WORKSPACE_TOML = os.path.join(PROJECT, "Cargo.toml")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Workspace frissítése az új crate-tel
ws = pathlib.Path(WORKSPACE_TOML).read_text()
if "athlesia-structure" not in ws:
    ws = ws.replace("\n]", ',\n    "crates/athlesia-structure"\n]', 1)
    write_file(WORKSPACE_TOML, ws)
    print("[1] Workspace frissítve az athlesia-structure crate-tel.")
else:
    print("[1] athlesia-structure már szerepel a workspace-ben.")

# 2. Structure crate létrehozása
os.makedirs(os.path.join(STRUCT_DIR, "src"), exist_ok=True)
os.makedirs(os.path.join(STRUCT_DIR, "tests"), exist_ok=True)

write_file(os.path.join(STRUCT_DIR, "Cargo.toml"), '''[package]
name = "athlesia-structure"
version = "0.1.0"
edition = "2021"

[dependencies]
athlesia-types = { path = "../athlesia-types" }
athlesia-executor = { path = "../athlesia-executor" }
''')

write_file(os.path.join(STRUCT_DIR, "src", "lib.rs"), r'''
use athlesia_types::{Grid, PrimName, Params};
use athlesia_executor::apply_primitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockDecomposition {
    pub block_rows: usize,
    pub block_cols: usize,
    pub block_width: usize,
    pub block_height: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransformId {
    Identity,
    Rot90,
    Rot180,
    Rot270,
    ReflectH,
    ReflectV,
}

#[derive(Debug, Clone)]
pub struct MetaGrid {
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<Option<TransformId>>,
}

pub struct TargetDecomposer;

impl TargetDecomposer {
    pub fn decompose_dimensions(input: &Grid, target: &Grid) -> Option<BlockDecomposition> {
        if input.width == 0 || input.height == 0 {
            return None;
        }
        let block_cols = target.width as usize / input.width as usize;
        let block_rows = target.height as usize / input.height as usize;
        if block_cols == 0 || block_rows == 0 {
            return None;
        }
        if target.width as usize % input.width as usize != 0
            || target.height as usize % input.height as usize != 0
        {
            return None;
        }
        Some(BlockDecomposition {
            block_rows,
            block_cols,
            block_width: input.width as usize,
            block_height: input.height as usize,
        })
    }

    fn extract_block(target: &Grid, decomp: &BlockDecomposition, r: usize, c: usize) -> Grid {
        let mut block = Grid::new(decomp.block_width as u8, decomp.block_height as u8);
        let start_x = c * decomp.block_width;
        let start_y = r * decomp.block_height;
        for y in 0..decomp.block_height {
            for x in 0..decomp.block_width {
                if let Some(cell) = target.get((start_x + x) as i8, (start_y + y) as i8) {
                    block.set(x as i8, y as i8, cell);
                }
            }
        }
        block
    }

    fn match_block(input: &Grid, block: &Grid) -> Option<TransformId> {
        if *block == *input {
            return Some(TransformId::Identity);
        }

        let transforms = [
            (TransformId::Rot90, PrimName::Rotate90),
            (TransformId::Rot180, PrimName::Rotate180),
            (TransformId::Rot270, PrimName::Rotate270),
            (TransformId::ReflectH, PrimName::ReflectH),
            (TransformId::ReflectV, PrimName::ReflectV),
        ];

        for (id, prim) in transforms {
            let transformed = apply_primitive(input, &prim, &Params::None);
            if transformed == *block {
                return Some(id);
            }
        }

        None
    }

    pub fn decompose(&self, input: &Grid, target: &Grid) -> Option<MetaGrid> {
        let decomp = Self::decompose_dimensions(input, target)?;
        let mut cells = Vec::with_capacity(decomp.block_rows * decomp.block_cols);

        for r in 0..decomp.block_rows {
            for c in 0..decomp.block_cols {
                let block = Self::extract_block(target, &decomp, r, c);
                let transform = Self::match_block(input, &block);
                cells.push(transform);
            }
        }

        Some(MetaGrid {
            rows: decomp.block_rows,
            cols: decomp.block_cols,
            cells,
        })
    }
}
''')

print("[2] athlesia-structure lib.rs létrehozva.")

# 3. Tesztek
write_file(os.path.join(STRUCT_DIR, "tests", "structure_test.rs"), r'''
use athlesia_structure::{TargetDecomposer, TransformId};
use athlesia_types::{Grid, Color};

fn grid_from_rows(rows: Vec<Vec<u8>>) -> Grid {
    let height = rows.len() as u8;
    let width = if height > 0 { rows[0].len() as u8 } else { 0 };
    let mut cells = Vec::new();
    for row in &rows {
        for &c in row {
            cells.push(Color(c));
        }
    }
    Grid { width, height, cells }
}

#[test]
fn decompose_2x2_to_4x4_all_identity() {
    let input = grid_from_rows(vec![
        vec![1, 2],
        vec![3, 4],
    ]);
    let target = grid_from_rows(vec![
        vec![1,2,1,2],
        vec![3,4,3,4],
        vec![1,2,1,2],
        vec![3,4,3,4],
    ]);

    let decomposer = TargetDecomposer;
    let meta = decomposer.decompose(&input, &target).expect("Dekompozíció nem található");
    assert_eq!(meta.rows, 2);
    assert_eq!(meta.cols, 2);
    assert!(meta.cells.iter().all(|t| *t == Some(TransformId::Identity)));
}

#[test]
fn decompose_2x2_to_4x4_with_rotations() {
    let input = grid_from_rows(vec![
        vec![1, 2],
        vec![3, 4],
    ]);
    let target = grid_from_rows(vec![
        vec![1,2,2,4],
        vec![3,4,1,3],
        vec![2,1,4,3],
        vec![4,3,2,1],
    ]);

    let decomposer = TargetDecomposer;
    let meta = decomposer.decompose(&input, &target).expect("Dekompozíció nem található");
    assert_eq!(meta.rows, 2);
    assert_eq!(meta.cols, 2);

    let expected = vec![
        Some(TransformId::Identity),
        Some(TransformId::Rot90),
        Some(TransformId::ReflectH),
        Some(TransformId::Rot180),
    ];
    assert_eq!(meta.cells, expected);
}

#[test]
fn no_decomposition_when_dimensions_not_divisible() {
    let input = grid_from_rows(vec![vec![1, 2]]);
    let target = grid_from_rows(vec![vec![1, 2, 3]]);

    let decomposer = TargetDecomposer;
    assert!(decomposer.decompose(&input, &target).is_none());
}
''')

print("[3] athlesia-structure tesztek létrehozva.")

# 4. Tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-structure"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] athlesia-structure tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] athlesia-structure tesztek zöldek.")

# 5. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Add athlesia-structure crate with Target Decomposer and MetaGrid"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
