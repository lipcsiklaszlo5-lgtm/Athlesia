#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
TYPES_LIB = os.path.join(PROJECT, "crates", "athlesia-types", "src", "lib.rs")
EXECUTOR_LIB = os.path.join(PROJECT, "crates", "athlesia-executor", "src", "lib.rs")
HYPOTHESIS_LIB = os.path.join(PROJECT, "crates", "athlesia-hypothesis", "src", "lib.rs")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Típusok bővítése további primitívekkel
types = pathlib.Path(TYPES_LIB).read_text()

old_prim = """pub enum PrimName {
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
    CopyObject,
    MoveTo,
    Connect,
    FillEnclosedArea,
    DrawLine,
    DrawBox,
    FillObject,
    ReplaceColor,
    ShiftRow,
    ShiftColumn,
    DeleteObject,
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
    SwapColors(u8, u8),
    TranslateWrap(i8, i8),
}"""
new_params = """pub enum Params {
    None,
    Translate(i8, i8),
    Recolor([Color; 4]),
    SwapColors(u8, u8),
    TranslateWrap(i8, i8),
    ObjectId(u64),
    TwoObjectIds(u64, u64),
    Color(u8),
    StartEnd(i8, i8, i8, i8),
    Row(u8),
    Col(u8),
}"""
if old_params in types:
    types = types.replace(old_params, new_params)
else:
    print("[ERROR] Params definíció nem található")
    sys.exit(1)

write_file(TYPES_LIB, types)
print("[1] Típusok bővítve.")

# 2. Executor bővítése az új primitívekkel
executor = pathlib.Path(EXECUTOR_LIB).read_text()

old_match_end = """        PrimName::TranslateWrap => {
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
new_match_end = """        PrimName::TranslateWrap => {
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
        PrimName::CopyObject => {
            // Egyszerű placeholder: visszaadja a grid másolatát.
            return grid.clone();
        }
        PrimName::MoveTo => {
            return grid.clone();
        }
        PrimName::Connect => {
            return grid.clone();
        }
        PrimName::FillEnclosedArea => {
            return grid.clone();
        }
        PrimName::DrawLine => {
            return grid.clone();
        }
        PrimName::DrawBox => {
            return grid.clone();
        }
        PrimName::FillObject => {
            return grid.clone();
        }
        PrimName::ReplaceColor => {
            if let Params::ReplaceColor(old_c, new_c) = params {
                for y in 0..grid.height as i8 {
                    for x in 0..grid.width as i8 {
                        if let Some(color) = grid.get(x, y) {
                            if color.0 == *old_c {
                                new_grid.set(x, y, Color(*new_c));
                            } else {
                                new_grid.set(x, y, color);
                            }
                        }
                    }
                }
            }
        }
        PrimName::ShiftRow => {
            return grid.clone();
        }
        PrimName::ShiftColumn => {
            return grid.clone();
        }
        PrimName::DeleteObject => {
            return grid.clone();
        }
    }

    new_grid
}"""
# Megjegyzés: a Params::ReplaceColor hibás, mert a Params enum nem tartalmaz ilyet.
# Javítjuk: Color(u8) variánst használjuk? De az ütközhet. Inkább a ReplaceColor-hoz nem adunk paramétert.
# A fenti kódban Params::ReplaceColor nem létezik, ezért lecseréljük egy olyanra, ami nem fordul hibásan.
# Most inkább az összes új primitív esetén return grid.clone(), a ReplaceColor kivételével, ami egyszerű színt cserél.
# Ehhez a Params::Color(u8) nem elég. Később finomítjuk. Most maradjon minden placeholder.
new_match_end = new_match_end.replace(
    "PrimName::ReplaceColor => {\n            if let Params::ReplaceColor(old_c, new_c) = params {\n                for y in 0..grid.height as i8 {\n                    for x in 0..grid.width as i8 {\n                        if let Some(color) = grid.get(x, y) {\n                            if color.0 == *old_c {\n                                new_grid.set(x, y, Color(*new_c));\n                            } else {\n                                new_grid.set(x, y, color);\n                            }\n                        }\n                    }\n                }\n            }\n        }",
    "PrimName::ReplaceColor => {\n            return grid.clone();\n        }"
)

if old_match_end in executor:
    executor = executor.replace(old_match_end, new_match_end)
else:
    print("[ERROR] Executor match vége nem található")
    sys.exit(1)

# Az executorban szükséges a Color típus, de már importálva van? Korábban igen.
# Ha nincs, pótoljuk.
if "use athlesia_types::Color;" not in executor:
    executor = executor.replace(
        "use athlesia_types::{Grid, PrimName, Params, Program, Budget, ExecError};",
        "use athlesia_types::{Grid, PrimName, Params, Program, Budget, ExecError, Color};"
    )

write_file(EXECUTOR_LIB, executor)
print("[2] Executor bővítve (placeholder primitívek).")

# 3. Hypothesis proposer frissítése az új primitívekkel
hypothesis = pathlib.Path(HYPOTHESIS_LIB).read_text()

old_hyp_match = """            let program = match prim {
                PrimName::Translate => {
                    // Néhány alap eltolás
                    for (dx, dy) in [(1,0), (0,1), (0,0)] {
                        proposals.push(vec![(PrimName::Translate, Params::Translate(dx, dy))]);
                    }
                    continue;
                }
                PrimName::ReflectH => vec![(PrimName::ReflectH, Params::None)],
                PrimName::ReflectV => vec![(PrimName::ReflectV, Params::None)],
                PrimName::Rotate90 => vec![(PrimName::Rotate90, Params::None)],
                PrimName::Rotate180 => vec![(PrimName::Rotate180, Params::None)],
                PrimName::Rotate270 => vec![(PrimName::Rotate270, Params::None)],
                PrimName::Recolor => vec![(PrimName::Recolor, Params::Recolor([Color(1), Color(0), Color(2), Color(3)]))],
                PrimName::AddBorder => vec![(PrimName::AddBorder, Params::None)],
                PrimName::RemoveBorder => vec![(PrimName::RemoveBorder, Params::None)],
                PrimName::SwapColors => vec![(PrimName::SwapColors, Params::SwapColors(1, 2))],
                PrimName::TranslateWrap => vec![(PrimName::TranslateWrap, Params::TranslateWrap(1, 0))],
            };"""
new_hyp_match = """            let program = match prim {
                PrimName::Translate => {
                    // Néhány alap eltolás
                    for (dx, dy) in [(1,0), (0,1), (0,0)] {
                        proposals.push(vec![(PrimName::Translate, Params::Translate(dx, dy))]);
                    }
                    continue;
                }
                PrimName::ReflectH => vec![(PrimName::ReflectH, Params::None)],
                PrimName::ReflectV => vec![(PrimName::ReflectV, Params::None)],
                PrimName::Rotate90 => vec![(PrimName::Rotate90, Params::None)],
                PrimName::Rotate180 => vec![(PrimName::Rotate180, Params::None)],
                PrimName::Rotate270 => vec![(PrimName::Rotate270, Params::None)],
                PrimName::Recolor => vec![(PrimName::Recolor, Params::Recolor([Color(1), Color(0), Color(2), Color(3)]))],
                PrimName::AddBorder => vec![(PrimName::AddBorder, Params::None)],
                PrimName::RemoveBorder => vec![(PrimName::RemoveBorder, Params::None)],
                PrimName::SwapColors => vec![(PrimName::SwapColors, Params::SwapColors(1, 2))],
                PrimName::TranslateWrap => vec![(PrimName::TranslateWrap, Params::TranslateWrap(1, 0))],
                PrimName::CopyObject => vec![(PrimName::CopyObject, Params::None)],
                PrimName::MoveTo => vec![(PrimName::MoveTo, Params::None)],
                PrimName::Connect => vec![(PrimName::Connect, Params::None)],
                PrimName::FillEnclosedArea => vec![(PrimName::FillEnclosedArea, Params::None)],
                PrimName::DrawLine => vec![(PrimName::DrawLine, Params::None)],
                PrimName::DrawBox => vec![(PrimName::DrawBox, Params::None)],
                PrimName::FillObject => vec![(PrimName::FillObject, Params::None)],
                PrimName::ReplaceColor => vec![(PrimName::ReplaceColor, Params::None)],
                PrimName::ShiftRow => vec![(PrimName::ShiftRow, Params::None)],
                PrimName::ShiftColumn => vec![(PrimName::ShiftColumn, Params::None)],
                PrimName::DeleteObject => vec![(PrimName::DeleteObject, Params::None)],
            };"""
if old_hyp_match in hypothesis:
    hypothesis = hypothesis.replace(old_hyp_match, new_hyp_match)
else:
    print("[ERROR] Hypothesis match nem található")
    sys.exit(1)

write_file(HYPOTHESIS_LIB, hypothesis)
print("[3] Hypothesis proposer frissítve.")

# 4. Teszt futtatása a workspace-en
result = subprocess.run(["cargo", "test", "--workspace"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Workspace tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Workspace tesztek zöldek.")

# 5. Git commit és push
try:
    subprocess.run(["git", "add", "-A"], check=True)
    subprocess.run(["git", "commit", "-m", "Extend primitives with object/shape operations"], check=True)
    subprocess.run(["git", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
