#!/usr/bin/env python3
import re, pathlib, subprocess, sys

# 1. Synthesis Engine lib.rs frissítése
p = pathlib.Path("crates/athlesia-synthesis/src/lib.rs")
s = p.read_text()

# Töröljük a Tile és RepeatGrid ágakat a generate_primitives-ből,
# mert azokat mostantól a synthesize dinamikusan kezeli.
s = s.replace(
    """PrimitiveTemplate::Tile => {
            vec![(PrimName::Tile, Params::None)]
        }
""",
    ""
)
s = s.replace(
    """PrimitiveTemplate::RepeatGrid => {
            vec![(PrimName::RepeatGrid, Params::None)]
        }
""",
    ""
)

# Kicseréljük a synthesize függvényt, hogy a dimenzióváltó primitíveket
# induktívan, a bemenet/cél méretarányából próbálja ki.
synthesize_start = s.find("pub fn synthesize")
if synthesize_start == -1:
    print("[ERROR] synthesize függvény nem található")
    sys.exit(1)

# Megkeressük a függvény végét: a következő '}\n\n' vagy a fájl végét.
end_marker = s.find("\n}\n", synthesize_start)
if end_marker == -1:
    end_marker = len(s)
else:
    end_marker += 3  # a záró kapcsos is része legyen

new_synthesize = '''pub fn synthesize(input: &Grid, target: &Grid, templates: &[PrimitiveTemplate]) -> Option<Program> {
    for template in templates {
        for (prim, params) in generate_primitives(*template) {
            let program = vec![(prim, params)];
            let mut budget = Budget { max_steps: 1, max_depth: 100 };
            if let Ok(output) = run_program(&program, input, &mut budget) {
                if output == *target {
                    return Some(program);
                }
            }
        }
    }

    // Dimenzióváltó primitívek induktív kipróbálása.
    // Például 3x3 -> 9x9 esetén RepeatGrid(3) vagy Tile(3).
    if input.width > 0 && input.height > 0 {
        let w_ratio = target.width / input.width;
        let h_ratio = target.height / input.height;
        if target.width % input.width == 0
            && target.height % input.height == 0
            && w_ratio == h_ratio
            && w_ratio > 0
        {
            let k = w_ratio as usize;
            let dim_programs = [
                vec![(PrimName::RepeatGrid, Params::RepeatGrid(k))],
                vec![(PrimName::Tile, Params::Tile(k))],
            ];
            for program in dim_programs.iter() {
                let mut budget = Budget { max_steps: 1, max_depth: 100 };
                if let Ok(output) = run_program(program, input, &mut budget) {
                    if output == *target {
                        return Some(program.clone());
                    }
                }
            }
        }
    }

    None
}
'''

s = s[:synthesize_start] + new_synthesize + s[end_marker:]
p.write_text(s)
print("[1] Synthesis Engine synthesize függvény frissítve dimenzióváltó primitívekkel.")

# 2. Új tesztfájl létrehozása
test_content = r'''
use athlesia_synthesis::synthesize;
use athlesia_types::{Grid, Color, PrimName, Params, Program};

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
fn synthesizes_repeat_grid_3x3_to_9x9() {
    let input = grid_from_rows(vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
    ]);
    let target = grid_from_rows(vec![
        vec![1,2,3,1,2,3,1,2,3],
        vec![4,5,6,4,5,6,4,5,6],
        vec![7,8,9,7,8,9,7,8,9],
        vec![1,2,3,1,2,3,1,2,3],
        vec![4,5,6,4,5,6,4,5,6],
        vec![7,8,9,7,8,9,7,8,9],
        vec![1,2,3,1,2,3,1,2,3],
        vec![4,5,6,4,5,6,4,5,6],
        vec![7,8,9,7,8,9,7,8,9],
    ]);

    let program = synthesize(&input, &target, &[]).expect("Meg kell találni a RepeatGrid(3)-at");
    assert_eq!(
        program,
        vec![(PrimName::RepeatGrid, Params::RepeatGrid(3))]
    );
}

#[test]
fn synthesizes_tile_2x2_to_4x4() {
    let input = grid_from_rows(vec![vec![1, 0], vec![0, 1]]);
    let target = grid_from_rows(vec![
        vec![1,1,0,0],
        vec![1,1,0,0],
        vec![0,0,1,1],
        vec![0,0,1,1],
    ]);

    let program = synthesize(&input, &target, &[]).expect("Meg kell találni a Tile(2)-t");
    assert_eq!(
        program,
        vec![(PrimName::Tile, Params::Tile(2))]
    );
}
'''
pathlib.Path("crates/athlesia-synthesis/tests/dimension_test.rs").write_text(test_content)
print("[2] Új dimenzióváltó tesztek létrehozva.")

# 3. Tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-synthesis", "--test", "dimension_test"],
    capture_output=True,
    text=True,
    check=False
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Dimenzióváltó tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Dimenzióváltó tesztek zöldek.")

# 4. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Add inductive dimension-changing primitives to Synthesis Engine"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
