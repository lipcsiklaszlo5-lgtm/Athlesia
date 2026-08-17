#!/usr/bin/env python3
import os, subprocess, sys, pathlib

def ensure_rust():
    cargo_bin = os.path.expanduser("~/.cargo/bin")
    os.environ["PATH"] = cargo_bin + os.pathsep + os.environ.get("PATH", "")
    try:
        subprocess.run(["cargo", "--version"], check=True, capture_output=True)
        print("[INFO] cargo már elérhető.")
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("[INFO] Rust/cargo nincs telepítve, telepítem...")
        subprocess.run("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y", shell=True, check=True)
        cargo_bin = os.path.expanduser("~/.cargo/bin")
        os.environ["PATH"] = cargo_bin + os.pathsep + os.environ.get("PATH", "")
        subprocess.run(["cargo", "--version"], check=True)
        print("[INFO] cargo telepítve.")

def create_workspace(base_dir):
    project = os.path.join(base_dir, "athlesia")
    os.makedirs(project, exist_ok=True)
    os.makedirs(os.path.join(project, "crates", "athlesia-types", "src"), exist_ok=True)
    os.makedirs(os.path.join(project, "crates", "athlesia-executor", "src"), exist_ok=True)
    os.makedirs(os.path.join(project, "crates", "athlesia-executor", "tests", "fixtures"), exist_ok=True)
    return project

WORKSPACE_CARGO_TOML = '''[workspace]
members = ["crates/athlesia-types", "crates/athlesia-executor"]
[workspace.package]
version = "0.1.0"
edition = "2021"
'''

TYPES_CARGO_TOML = '''[package]
name = "athlesia-types"
version = "0.1.0"
edition = "2021"
[dependencies]
'''

TYPES_LIB_RS = '''pub const GRID_SIZE: usize = 5;
pub const N_COLORS: u8 = 4;
pub type Color = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grid { pub cells: [[Color; GRID_SIZE]; GRID_SIZE] }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimName { Translate, ReflectH, ReflectV, Rotate90, Recolor }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Params { None, Translate(i8, i8), Recolor([Color; 4]) }

pub type Program = Vec<(PrimName, Params)>;

#[derive(Debug, Clone, Copy)]
pub struct Budget { pub max_steps: u64 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecError { BudgetExceeded }
'''

EXECUTOR_CARGO_TOML = '''[package]
name = "athlesia-executor"
version = "0.1.0"
edition = "2021"
[dependencies]
athlesia-types = { path = "../athlesia-types" }
[dev-dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
'''

EXECUTOR_LIB_RS = '''use athlesia_types::{Grid, PrimName, Params, Program, Budget, ExecError, GRID_SIZE};

pub fn apply_primitive(grid: &Grid, name: &PrimName, params: &Params) -> Grid {
    let mut new_grid = Grid { cells: [[0; GRID_SIZE]; GRID_SIZE] };
    match name {
        PrimName::Translate => {
            if let Params::Translate(dx, dy) = params {
                let (dx, dy) = (*dx as i8, *dy as i8);
                for i in 0..GRID_SIZE as i8 {
                    for j in 0..GRID_SIZE as i8 {
                        let (ni, nj) = (i + dy, j + dx);
                        if ni >= 0 && ni < GRID_SIZE as i8 && nj >= 0 && nj < GRID_SIZE as i8 {
                            new_grid.cells[ni as usize][nj as usize] = grid.cells[i as usize][j as usize];
                        }
                    }
                }
            }
        }
        PrimName::ReflectH => {
            for i in 0..GRID_SIZE {
                for j in 0..GRID_SIZE {
                    new_grid.cells[i][j] = grid.cells[i][GRID_SIZE - 1 - j];
                }
            }
        }
        PrimName::ReflectV => {
            for i in 0..GRID_SIZE {
                for j in 0..GRID_SIZE {
                    new_grid.cells[i][j] = grid.cells[GRID_SIZE - 1 - i][j];
                }
            }
        }
        PrimName::Rotate90 => {
            for i in 0..GRID_SIZE {
                for j in 0..GRID_SIZE {
                    new_grid.cells[i][j] = grid.cells[j][GRID_SIZE - 1 - i];
                }
            }
        }
        PrimName::Recolor => {
            if let Params::Recolor(perm) = params {
                for i in 0..GRID_SIZE {
                    for j in 0..GRID_SIZE {
                        let old = grid.cells[i][j] as usize;
                        new_grid.cells[i][j] = perm[old];
                    }
                }
            }
        }
    }
    new_grid
}

pub fn run_program(program: &Program, input: &Grid, budget: &mut Budget) -> Result<Grid, ExecError> {
    let mut current = *input;
    for (name, params) in program {
        if budget.max_steps == 0 { return Err(ExecError::BudgetExceeded); }
        current = apply_primitive(&current, name, params);
        budget.max_steps -= 1;
    }
    Ok(current)
}
'''

GOLDEN_VECTORS_JSON = '''[{"name": "translate_basic", "program": [["translate", [1, -1]]], "input": [[1, 0, 1, 0, 3], [2, 1, 3, 3, 3], [2, 3, 1, 0, 2], [1, 2, 2, 2, 1], [0, 2, 2, 0, 2]], "expected_output": [[0, 2, 1, 3, 3], [0, 2, 3, 1, 0], [0, 1, 2, 2, 2], [0, 0, 2, 2, 0], [0, 0, 0, 0, 0]]}, {"name": "translate_identity", "program": [["translate", [0, 0]]], "input": [[1, 0, 1, 0, 3], [2, 1, 3, 3, 3], [2, 3, 1, 0, 2], [1, 2, 2, 2, 1], [0, 2, 2, 0, 2]], "expected_output": [[1, 0, 1, 0, 3], [2, 1, 3, 3, 3], [2, 3, 1, 0, 2], [1, 2, 2, 2, 1], [0, 2, 2, 0, 2]]}, {"name": "reflect_h_basic", "program": [["reflect_h", []]], "input": [[1, 0, 1, 0, 3], [2, 1, 3, 3, 3], [2, 3, 1, 0, 2], [1, 2, 2, 2, 1], [0, 2, 2, 0, 2]], "expected_output": [[3, 0, 1, 0, 1], [3, 3, 3, 1, 2], [2, 0, 1, 3, 2], [1, 2, 2, 2, 1], [2, 0, 2, 2, 0]]}, {"name": "reflect_v_basic", "program": [["reflect_v", []]], "input": [[1, 0, 1, 0, 3], [2, 1, 3, 3, 3], [2, 3, 1, 0, 2], [1, 2, 2, 2, 1], [0, 2, 2, 0, 2]], "expected_output": [[0, 2, 2, 0, 2], [1, 2, 2, 2, 1], [2, 3, 1, 0, 2], [2, 1, 3, 3, 3], [1, 0, 1, 0, 3]]}, {"name": "rotate90_basic", "program": [["rotate90", []]], "input": [[1, 0, 1, 0, 3], [2, 1, 3, 3, 3], [2, 3, 1, 0, 2], [1, 2, 2, 2, 1], [0, 2, 2, 0, 2]], "expected_output": [[3, 3, 2, 1, 2], [0, 3, 0, 2, 0], [1, 3, 1, 2, 2], [0, 1, 3, 2, 2], [1, 2, 2, 1, 0]]}, {"name": "recolor_basic", "program": [["recolor", [2, 3, 0, 1]]], "input": [[1, 0, 1, 0, 3], [2, 1, 3, 3, 3], [2, 3, 1, 0, 2], [1, 2, 2, 2, 1], [0, 2, 2, 0, 2]], "expected_output": [[3, 2, 3, 2, 1], [0, 3, 1, 1, 1], [0, 1, 3, 2, 0], [3, 0, 0, 0, 3], [2, 0, 0, 2, 0]]}, {"name": "composite_translate_recolor", "program": [["translate", [-1, 1]], ["recolor", [3, 1, 2, 0]]], "input": [[3, 3, 2, 0, 0], [0, 2, 2, 2, 2], [3, 3, 3, 0, 2], [3, 1, 1, 1, 0], [1, 3, 0, 3, 3]], "expected_output": [[3, 3, 3, 3, 3], [0, 2, 3, 3, 3], [2, 2, 2, 2, 3], [0, 0, 3, 2, 3], [1, 1, 1, 3, 3]]}, {"name": "composite_reflect_rotate", "program": [["reflect_h", []], ["rotate90", []]], "input": [[3, 3, 2, 0, 0], [0, 2, 2, 2, 2], [3, 3, 3, 0, 2], [3, 1, 1, 1, 0], [1, 3, 0, 3, 3]], "expected_output": [[3, 0, 3, 3, 1], [3, 2, 3, 1, 3], [2, 2, 3, 1, 0], [0, 2, 0, 1, 3], [0, 2, 2, 0, 3]]}, {"name": "composite_three_step", "program": [["rotate90", []], ["translate", [1, 0]], ["recolor", [1, 0, 3, 2]]], "input": [[3, 3, 2, 0, 0], [0, 2, 2, 2, 2], [3, 3, 3, 0, 2], [3, 1, 1, 1, 0], [1, 3, 0, 3, 3]], "expected_output": [[1, 1, 3, 3, 1], [1, 1, 3, 1, 0], [1, 3, 3, 2, 0], [1, 2, 3, 2, 0], [1, 2, 1, 2, 2]]}, {"name": "edge_empty_translate", "program": [["translate", [1, 1]]], "input": [[0, 0, 0, 0, 0], [0, 0, 0, 0, 0], [0, 0, 0, 0, 0], [0, 0, 0, 0, 0], [0, 0, 0, 0, 0]], "expected_output": [[0, 0, 0, 0, 0], [0, 0, 0, 0, 0], [0, 0, 0, 0, 0], [0, 0, 0, 0, 0], [0, 0, 0, 0, 0]]}, {"name": "edge_full_recolor", "program": [["recolor", [3, 2, 1, 0]]], "input": [[3, 3, 3, 3, 3], [3, 3, 3, 3, 3], [3, 3, 3, 3, 3], [3, 3, 3, 3, 3], [3, 3, 3, 3, 3]], "expected_output": [[0, 0, 0, 0, 0], [0, 0, 0, 0, 0], [0, 0, 0, 0, 0], [0, 0, 0, 0, 0], [0, 0, 0, 0, 0]]}, {"name": "edge_empty_rotate", "program": [["rotate90", []]], "input": [[0, 0, 0, 0, 0], [0, 0, 0, 0, 0], [0, 0, 0, 0, 0], [0, 0, 0, 0, 0], [0, 0, 0, 0, 0]], "expected_output": [[0, 0, 0, 0, 0], [0, 0, 0, 0, 0], [0, 0, 0, 0, 0], [0, 0, 0, 0, 0], [0, 0, 0, 0, 0]]}]'''

GOLDEN_TEST_RS = '''use athlesia_executor::{run_program, Budget};
use athlesia_types::{Grid, PrimName, Params, GRID_SIZE};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct GoldenVector {
    name: String,
    program: Vec<(String, serde_json::Value)>,
    input: Vec<Vec<u8>>,
    expected_output: Vec<Vec<u8>>,
}

fn grid_from_vec(v: &[Vec<u8>]) -> Grid {
    let mut cells = [[0u8; GRID_SIZE]; GRID_SIZE];
    for i in 0..GRID_SIZE { for j in 0..GRID_SIZE { cells[i][j] = v[i][j]; } }
    Grid { cells }
}

#[test]
fn golden_vectors() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/golden_vectors.json");
    let content = fs::read_to_string(path).expect("Failed to read golden_vectors.json");
    let cases: Vec<GoldenVector> = serde_json::from_str(&content).expect("Invalid JSON");

    let mut failed = 0;
    for case in &cases {
        let program: Vec<(PrimName, Params)> = case.program.iter().map(|(op, params)| {
            let prim = match op.as_str() {
                "translate" => PrimName::Translate,
                "reflect_h" => PrimName::ReflectH,
                "reflect_v" => PrimName::ReflectV,
                "rotate90" => PrimName::Rotate90,
                "recolor" => PrimName::Recolor,
                other => panic!("Unknown primitive: {}", other),
            };
            let parsed_params = match prim {
                PrimName::Translate => {
                    let arr = params.as_array().expect("Translate params should be array");
                    Params::Translate(arr[0].as_i64().unwrap() as i8, arr[1].as_i64().unwrap() as i8)
                }
                PrimName::Recolor => {
                    let arr = params.as_array().expect("Recolor params should be array");
                    let mut perm = [0u8; 4];
                    for (i, v) in arr.iter().enumerate() { perm[i] = v.as_u64().unwrap() as u8; }
                    Params::Recolor(perm)
                }
                _ => Params::None,
            };
            (prim, parsed_params)
        }).collect();

        let input_grid = grid_from_vec(&case.input);
        let expected_grid = grid_from_vec(&case.expected_output);
        let mut budget = Budget { max_steps: 1000 };

        let result = run_program(&program, &input_grid, &mut budget);
        let success = match &result {
            Ok(output) => *output == expected_grid,
            Err(_) => false,
        };

        if !success {
            failed += 1;
            println!("FAIL: {}", case.name);
            match &result {
                Ok(output) => {
                    println!("Expected:\n{:?}", expected_grid);
                    println!("Got:\n{:?}", output);
                }
                Err(e) => println!("Error: {:?}", e),
            }
        }
    }
    assert_eq!(failed, 0, "{} golden test(s) failed", failed);
}
'''

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

def main():
    ensure_rust()
    base = os.getcwd()
    project = create_workspace(base)

    write_file(os.path.join(project, "Cargo.toml"), WORKSPACE_CARGO_TOML)
    write_file(os.path.join(project, "crates", "athlesia-types", "Cargo.toml"), TYPES_CARGO_TOML)
    write_file(os.path.join(project, "crates", "athlesia-types", "src", "lib.rs"), TYPES_LIB_RS)
    write_file(os.path.join(project, "crates", "athlesia-executor", "Cargo.toml"), EXECUTOR_CARGO_TOML)
    write_file(os.path.join(project, "crates", "athlesia-executor", "src", "lib.rs"), EXECUTOR_LIB_RS)
    write_file(os.path.join(project, "crates", "athlesia-executor", "tests", "fixtures", "golden_vectors.json"), GOLDEN_VECTORS_JSON)
    write_file(os.path.join(project, "crates", "athlesia-executor", "tests", "golden_test.rs"), GOLDEN_TEST_RS)

    print("[INFO] Fájlok létrehozva. Futtatom a teszteket...")
    os.chdir(project)
    result = subprocess.run(["cargo", "test", "-p", "athlesia-executor"], capture_output=True, text=True, check=False)
    sys.stdout.write(result.stdout)
    sys.stderr.write(result.stderr)
    if result.returncode == 0:
        print("\n[SUCCESS] Minden golden teszt zöld.")
    else:
        print("\n[FAILURE] Tesztek nem mentek át.")

if __name__ == "__main__":
    main()
