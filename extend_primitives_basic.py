#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
TYPES_LIB = os.path.join(PROJECT, "crates", "athlesia-types", "src", "lib.rs")
EXECUTOR_LIB = os.path.join(PROJECT, "crates", "athlesia-executor", "src", "lib.rs")
GOLDEN_TEST = os.path.join(PROJECT, "crates", "athlesia-executor", "tests", "golden_test.rs")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Típusok bővítése: PrimName és Params
types = pathlib.Path(TYPES_LIB).read_text()

old_prim = """pub enum PrimName {
    Translate,
    ReflectH,
    ReflectV,
    Rotate90,
    Recolor,
}"""
new_prim = """pub enum PrimName {
    Translate,
    ReflectH,
    ReflectV,
    Rotate90,
    Rotate180,
    Rotate270,
    Recolor,
    AddBorder,
    RemoveBorder,
    SwapColors,
    TranslateWrap,
}"""
if old_prim in types:
    types = types.replace(old_prim, new_prim)
else:
    print("[ERROR] PrimName definíció nem található")
    sys.exit(1)

old_params = """pub enum Params {
    None,
    Translate(i8, i8),
    Recolor([Color; 4]),
}"""
new_params = """pub enum Params {
    None,
    Translate(i8, i8),
    Recolor([Color; 4]),
    SwapColors(u8, u8),
    TranslateWrap(i8, i8),
}"""
if old_params in types:
    types = types.replace(old_params, new_params)
else:
    print("[ERROR] Params definíció nem található")
    sys.exit(1)

write_file(TYPES_LIB, types)
print("[1] Típusok bővítve.")

# 2. Executor apply_primitive bővítése
executor = pathlib.Path(EXECUTOR_LIB).read_text()

# Importok: Color már használva van, de Params bővült, ezért nincs új import szükséges.

# Beszúrás a match name blokkba, a Recolor ág után
old_match_end = """        PrimName::Recolor => {
            if let Params::Recolor(perm) = params {
                for y in 0..grid.height as i8 {
                    for x in 0..grid.width as i8 {
                        if let Some(color) = grid.get(x, y) {
                            let new_color = perm[color.0 as usize];
                            new_grid.set(x, y, new_color);
                        }
                    }
                }
            }
        }
    }

    new_grid
}"""
new_match_end = """        PrimName::Recolor => {
            if let Params::Recolor(perm) = params {
                for y in 0..grid.height as i8 {
                    for x in 0..grid.width as i8 {
                        if let Some(color) = grid.get(x, y) {
                            let new_color = perm[color.0 as usize];
                            new_grid.set(x, y, new_color);
                        }
                    }
                }
            }
        }
        PrimName::Rotate180 => {
            for y in 0..grid.height as i8 {
                for x in 0..grid.width as i8 {
                    if let Some(color) = grid.get(x, y) {
                        new_grid.set(grid.width as i8 - 1 - x, grid.height as i8 - 1 - y, color);
                    }
                }
            }
        }
        PrimName::Rotate270 => {
            // 270 fok CCW = 90 fok CW
            // new[x][y] = old[height-1-y][x]  (CW)
            for y in 0..grid.height as i8 {
                for x in 0..grid.width as i8 {
                    if let Some(color) = grid.get(x, y) {
                        new_grid.set(grid.height as i8 - 1 - y, x, color);
                    }
                }
            }
        }
        PrimName::AddBorder => {
            let new_width = grid.width + 2;
            let new_height = grid.height + 2;
            let mut bordered = Grid::new(new_width, new_height);
            for y in 0..grid.height as i8 {
                for x in 0..grid.width as i8 {
                    if let Some(color) = grid.get(x, y) {
                        bordered.set(x + 1, y + 1, color);
                    }
                }
            }
            return bordered;
        }
        PrimName::RemoveBorder => {
            if grid.width < 3 || grid.height < 3 {
                return grid.clone();
            }
            let new_width = grid.width - 2;
            let new_height = grid.height - 2;
            let mut cropped = Grid::new(new_width, new_height);
            for y in 1..grid.height as i8 - 1 {
                for x in 1..grid.width as i8 - 1 {
                    if let Some(color) = grid.get(x, y) {
                        cropped.set(x - 1, y - 1, color);
                    }
                }
            }
            return cropped;
        }
        PrimName::SwapColors => {
            if let Params::SwapColors(c1, c2) = params {
                for y in 0..grid.height as i8 {
                    for x in 0..grid.width as i8 {
                        if let Some(color) = grid.get(x, y) {
                            let new_color = if color.0 == *c1 {
                                Color(*c2)
                            } else if color.0 == *c2 {
                                Color(*c1)
                            } else {
                                color
                            };
                            new_grid.set(x, y, new_color);
                        }
                    }
                }
            }
        }
        PrimName::TranslateWrap => {
            if let Params::TranslateWrap(dx, dy) = params {
                let dx = *dx as i8;
                let dy = *dy as i8;
                for y in 0..grid.height as i8 {
                    for x in 0..grid.width as i8 {
                        if let Some(color) = grid.get(x, y) {
                            let nx = (x + dx).rem_euclid(grid.width as i8);
                            let ny = (y + dy).rem_euclid(grid.height as i8);
                            new_grid.set(nx, ny, color);
                        }
                    }
                }
            }
        }
    }

    new_grid
}"""
if old_match_end in executor:
    executor = executor.replace(old_match_end, new_match_end)
else:
    print("[ERROR] Executor match vége nem található")
    sys.exit(1)

write_file(EXECUTOR_LIB, executor)
print("[2] Executor bővítve.")

# 3. Golden test bővítése: új teszt vektorok
golden = pathlib.Path(GOLDEN_TEST).read_text()

# A meglévő grid_from_vec használható
# Új teszteket a golden_vectors tömbhöz adunk.
# De egyszerűbb külön tesztfájlt csinálni az új primitívekhez.
# Nem nyúlunk a meglévő JSON-hoz, hanem létrehozunk egy új integration tesztet.

new_test = r'''
use athlesia_executor::{apply_primitive, run_program};
use athlesia_types::{Grid, PrimName, Params, Budget, Color};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn rotate180_works() {
    let grid = build_grid([
        [1, 2, 0, 0, 0],
        [3, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 4, 0],
        [0, 0, 0, 0, 5],
    ]);
    let result = apply_primitive(&grid, &PrimName::Rotate180, &Params::None);

    let expected = build_grid([
        [5, 0, 0, 0, 0],
        [0, 4, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 3],
        [0, 0, 0, 2, 1],
    ]);

    assert_eq!(result, expected);
}

#[test]
fn rotate270_works() {
    let grid = build_grid([
        [1, 2, 0, 0, 0],
        [3, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 4, 0],
        [0, 0, 0, 0, 5],
    ]);
    let result = apply_primitive(&grid, &PrimName::Rotate270, &Params::None);

    let expected = build_grid([
        [0, 0, 0, 3, 1],
        [0, 0, 0, 0, 2],
        [0, 0, 0, 0, 0],
        [0, 5, 0, 4, 0],
        [0, 0, 0, 0, 0],
    ]);

    assert_eq!(result, expected);
}

#[test]
fn add_border_increases_size() {
    let grid = build_grid([[1; 5]; 5]);
    let result = apply_primitive(&grid, &PrimName::AddBorder, &Params::None);
    assert_eq!(result.width, 7);
    assert_eq!(result.height, 7);
    assert_eq!(result.get(0, 0), Some(Color(0)));
    assert_eq!(result.get(1, 1), Some(Color(1)));
}

#[test]
fn remove_border_decreases_size() {
    let grid = build_grid([
        [0, 0, 0, 0, 0],
        [0, 1, 1, 1, 0],
        [0, 1, 1, 1, 0],
        [0, 1, 1, 1, 0],
        [0, 0, 0, 0, 0],
    ]);
    let result = apply_primitive(&grid, &PrimName::RemoveBorder, &Params::None);
    assert_eq!(result.width, 3);
    assert_eq!(result.height, 3);
    assert_eq!(result.get(0, 0), Some(Color(1)));
}

#[test]
fn swap_colors_works() {
    let grid = build_grid([
        [1, 2, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let result = apply_primitive(&grid, &PrimName::SwapColors, &Params::SwapColors(1, 2));

    let expected = build_grid([
        [2, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    assert_eq!(result, expected);
}

#[test]
fn translate_wrap_wraps_around() {
    let grid = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    // Eltolás jobbra eggyel: az 1-es a szélen kívülre kerülne, de ciklikusan visszajön balra.
    let result = apply_primitive(&grid, &PrimName::TranslateWrap, &Params::TranslateWrap(1, 0));

    let expected = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    // Igazából a TranslateWrap jobbra viszi, de a szélen nincs wrap, mert a (0,0)-ból (0,1) lesz.
    // Ez nem teszteli a wrap-et, mert nincs túlcsordulás.
    // Írjunk egy olyan tesztet, ahol az (4,0)-n van elem.
    let grid2 = build_grid([
        [0, 0, 0, 0, 1],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let result2 = apply_primitive(&grid2, &PrimName::TranslateWrap, &Params::TranslateWrap(1, 0));
    let expected2 = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    assert_eq!(result2, expected2);
}
'''
write_file(os.path.join(PROJECT, "crates", "athlesia-executor", "tests", "new_primitives_test.rs"), new_test)
print("[3] Új primitívek tesztje létrehozva.")

# 4. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-executor", "--test", "new_primitives_test"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Új primitívek tesztjei nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Új primitívek tesztjei zöldek.")

# 5. Git commit és push
try:
    subprocess.run(["git", "add", "-A"], check=True)
    subprocess.run(["git", "commit", "-m", "Extend primitives with rotate180, rotate270, borders, swap, wrap"], check=True)
    subprocess.run(["git", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
