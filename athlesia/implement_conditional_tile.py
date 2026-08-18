#!/usr/bin/env python3
import os, subprocess, sys, pathlib

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. PrimName: ConditionalTile hozzáadása
p = pathlib.Path("crates/athlesia-types/src/lib.rs")
s = p.read_text()
old_prim = "    RepeatGrid,\n    BlockMap,\n}"
new_prim = "    RepeatGrid,\n    BlockMap,\n    ConditionalTile,\n}"
if old_prim in s:
    s = s.replace(old_prim, new_prim)
    write_file(p, s)
    print("[1] ConditionalTile hozzáadva a PrimName-hez.")
else:
    print("[ERROR] PrimName végét nem találtam.")
    sys.exit(1)

# 2. Params: ConditionalTile paraméter hozzáadása
old_params = "    RepeatGrid(usize),\n    BlockMap(usize, usize, Vec<u8>),\n}"
new_params = "    RepeatGrid(usize),\n    BlockMap(usize, usize, Vec<u8>),\n    ConditionalTile,\n}"
if old_params in s:
    s = s.replace(old_params, new_params)
    write_file(p, s)
    print("[2] ConditionalTile paraméter hozzáadva.")
else:
    print("[ERROR] Params végét nem találtam.")
    sys.exit(1)

# 3. Executor: ConditionalTile implementációja
p = pathlib.Path("crates/athlesia-executor/src/lib.rs")
s = p.read_text()
old_match = "        PrimName::RepeatGrid => {"
new_match = """        PrimName::ConditionalTile => {
            // Az inputot maszkként használjuk: csak azokra a blokk-pozíciókra
            // helyezzük el az inputot, ahol a maszk cella nem háttérszín (0).
            // A kimenet mérete: input.width * input.width, input.height * input.height? 
            // Helyesebben: a maszk dimenziói határozzák meg a blokkok számát.
            // Itt egyszerű esetet valósítunk meg: a maszk = input, a minta = input.
            // A kimenet: (input.width * input.width) x (input.height * input.height)
            let tile_h = grid.height;
            let tile_w = grid.width;
            let out_h = tile_h * tile_h; // rows = input.height
            let out_w = tile_w * tile_w; // cols = input.width
            let mut out = Grid::new(out_w as u8, out_h as u8);

            for mask_y in 0..grid.height as i8 {
                for mask_x in 0..grid.width as i8 {
                    let mask_val = grid.get(mask_x, mask_y).unwrap_or(Color(0));
                    if mask_val != Color(0) {
                        // A minta (input) elhelyezése a blokk-pozícióra
                        let start_x = mask_x * tile_w as i8;
                        let start_y = mask_y * tile_h as i8;
                        for y in 0..grid.height as i8 {
                            for x in 0..grid.width as i8 {
                                if let Some(cell) = grid.get(x, y) {
                                    out.set(start_x + x, start_y + y, cell);
                                }
                            }
                        }
                    }
                }
            }
            return out;
        }
        PrimName::RepeatGrid => {"""
if old_match in s:
    s = s.replace(old_match, new_match)
    write_file(p, s)
    print("[3] ConditionalTile implementálva az Executorban.")
else:
    print("[ERROR] RepeatGrid blokk nem található az Executorban.")
    sys.exit(1)

# 4. Synthesis: ConditionalTile hozzáadása a dimenzióváltó próbálkozásokhoz
p = pathlib.Path("crates/athlesia-synthesis/src/lib.rs")
s = p.read_text()
old_dim = "    // Dimenzióváltó primitívek induktív kipróbálása."
new_dim = """    // ConditionalTile kipróbálása, ha a target mérete input.width * input.width, input.height * input.height
    if target.width == input.width.saturating_mul(input.width) &&
       target.height == input.height.saturating_mul(input.height)
    {
        let program = vec![(PrimName::ConditionalTile, Params::ConditionalTile)];
        let mut budget = Budget { max_steps: 1, max_depth: 100 };
        if let Ok(output) = run_program(&program, input, &mut budget) {
            if output == *target {
                return Some(program);
            }
        }
    }

    // Dimenzióváltó primitívek induktív kipróbálása."""
if old_dim in s:
    s = s.replace(old_dim, new_dim)
    write_file(p, s)
    print("[4] ConditionalTile hozzáadva a Synthesis Engine-hez.")
else:
    print("[ERROR] Dimenzióváltó szintézis blokk nem található.")
    sys.exit(1)

# 5. Új teszt a ConditionalTile-hoz
write_file("crates/athlesia-executor/tests/conditional_tile_test.rs", r'''
use athlesia_executor::apply_primitive;
use athlesia_types::{Grid, PrimName, Params};

fn build_grid(rows: [[u8; 3]; 3]) -> Grid {
    Grid::from_5x5([]) // placeholder, nem használjuk
}
''')

# A fenti teszt hibás, mert a Grid::from_5x5 5x5-ös, de a ConditionalTile 3x3 bemenetet vár.
# Javítjuk a tesztet egy rendes grid_from_rows függvénnyel.
write_file("crates/athlesia-executor/tests/conditional_tile_test.rs", r'''
use athlesia_executor::apply_primitive;
use athlesia_types::{Grid, Color, PrimName, Params};

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
fn conditional_tile_places_only_on_foreground() {
    // Bemenet: 2x2-es maszk, ahol a (0,0) és (1,1) nem háttér
    let mask = grid_from_rows(vec![
        vec![1, 0],
        vec![0, 1],
    ]);
    let output = apply_primitive(&mask, &PrimName::ConditionalTile, &Params::ConditionalTile);

    // Kimenet: 4x4-es rács, ahol a blokkok a maszk szerint helyezkednek el
    // (0,0)-nél a 2x2-es minta, (1,1)-nél a 2x2-es minta, a többi üres.
    let expected = grid_from_rows(vec![
        vec![1,0,0,0],
        vec![0,1,0,0],
        vec![0,0,1,0],
        vec![0,0,0,1],
    ]);

    assert_eq!(output, expected);
}
''')
print("[5] ConditionalTile teszt hozzáadva.")

# 6. Tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-executor", "--test", "conditional_tile_test"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] ConditionalTile tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] ConditionalTile tesztek zöldek.")

# 7. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Add ConditionalTile primitive for self-referential tiling"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
