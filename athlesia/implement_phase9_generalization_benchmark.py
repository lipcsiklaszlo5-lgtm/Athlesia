#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Generalization benchmark teszt létrehozása
test_code = r'''
use athlesia_core::CoreEngine;
use athlesia_types::{Grid, Color};

/// Egyszerű rács létrehozása sorvektorokból.
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
fn generalization_reduces_search_cost_for_reflect_h() {
    let mut engine = CoreEngine::new();

    // Első feladat: 3x3-as grid tükrözése vízszintesen.
    let input1 = grid_from_rows(vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
    ]);
    let target1 = grid_from_rows(vec![
        vec![3, 2, 1],
        vec![6, 5, 4],
        vec![9, 8, 7],
    ]);

    let (result1, steps1) = engine.solve_with_steps(&input1, &target1);
    assert!(result1.is_some(), "Az első feladatot meg kellett oldani");
    assert!(steps1 > 1, "Az első feladat keresést igényel, steps1={}", steps1);

    // Második feladat: 5x5-ös grid, ugyanaz a szabály (ReflectH).
    let input2 = grid_from_rows(vec![
        vec![1, 0, 2, 0, 3],
        vec![4, 5, 0, 6, 7],
        vec![0, 8, 9, 0, 1],
        vec![2, 3, 0, 4, 5],
        vec![6, 0, 7, 8, 0],
    ]);
    let target2 = grid_from_rows(vec![
        vec![3, 0, 2, 0, 1],
        vec![7, 6, 0, 5, 4],
        vec![1, 0, 9, 8, 0],
        vec![5, 4, 0, 3, 2],
        vec![0, 8, 7, 0, 6],
    ]);

    let (result2, steps2) = engine.solve_with_steps(&input2, &target2);
    assert!(result2.is_some(), "A második feladatot is meg kellett oldani");
    assert!(
        steps2 < steps1,
        "A második feladatnak kevesebb lépéssel kell megoldódnia: steps1={}, steps2={}",
        steps1,
        steps2
    );
}
'''
write_file("crates/athlesia-core/tests/generalization_benchmark.rs", test_code)
print("[1] generalization_benchmark.rs létrehozva.")

# 2. Core tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-core"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Generalization benchmark nem ment át.")
    sys.exit(1)

print("\n[SUCCESS] Phase 9 benchmark zöld.")

# 3. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 9: generalization benchmark shows reduced search cost after learning"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
