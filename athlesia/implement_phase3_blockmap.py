#!/usr/bin/env python3
import os, subprocess, sys, pathlib

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Executor: BlockMap primitív hozzáadása
p = pathlib.Path("crates/athlesia-executor/src/lib.rs")
s = p.read_text()

old_match = "        PrimName::RepeatGrid => {"
new_match = "        PrimName::BlockMap => {\n            // BlockMap: a targetet input-méretű blokkokra bontja, és minden blokkra\n            // a Paraméterben kapott transzformáció-azonosító listát alkalmazza.\n            // A Params::BlockMap tartalmazza a sorok/oszlopok számát és a transzformációkat.\n            if let Params::BlockMap(rows, cols, transforms) = params {\n                let block_h = grid.height;\n                let block_w = grid.width;\n                let out_h = block_h * (*rows as u8);\n                let out_w = block_w * (*cols as u8);\n                let mut out = Grid::new(out_w, out_h);\n\n                for r in 0..*rows {\n                    for c in 0..*cols {\n                        let idx = r * *cols + c;\n                        let transform_id = transforms.get(idx).copied().unwrap_or(0u8);\n                        let block = match transform_id {\n                            1 => apply_primitive(grid, &PrimName::Rotate90, &Params::None),\n                            2 => apply_primitive(grid, &PrimName::Rotate180, &Params::None),\n                            3 => apply_primitive(grid, &PrimName::Rotate270, &Params::None),\n                            4 => apply_primitive(grid, &PrimName::ReflectH, &Params::None),\n                            5 => apply_primitive(grid, &PrimName::ReflectV, &Params::None),\n                            _ => grid.clone(),\n                        };\n\n                        let start_x = (c as i8) * block_w as i8;\n                        let start_y = (r as i8) * block_h as i8;\n                        for y in 0..block_h as i8 {\n                            for x in 0..block_w as i8 {\n                                if let Some(cell) = block.get(x, y) {\n                                    out.set(start_x + x, start_y + y, cell);\n                                }\n                            }\n                        }\n                    }\n                }\n                return out;\n            }\n            return grid.clone();\n        }\n        PrimName::RepeatGrid => {"

if old_match in s:
    s = s.replace(old_match, new_match)
    write_file(p, s)
    print("[1] Executor BlockMap primitív hozzáadva.")
else:
    print("[ERROR] Nem találtam a RepeatGrid blokkot az executorban.")
    sys.exit(1)

# 2. Types: BlockMap paraméter hozzáadása
p = pathlib.Path("crates/athlesia-types/src/lib.rs")
s = p.read_text()

old_params = "    RepeatGrid(usize),\n}"
new_params = "    RepeatGrid(usize),\n    BlockMap(usize, usize, Vec<u8>),\n}"
if old_params in s:
    s = s.replace(old_params, new_params)
    write_file(p, s)
    print("[2] Types BlockMap paraméter hozzáadva.")
else:
    print("[ERROR] Nem találtam a Params enum végét.")
    sys.exit(1)

# 3. Synthesis Engine: BlockMap generálása MetaGrid-ből
p = pathlib.Path("crates/athlesia-synthesis/src/lib.rs")
s = p.read_text()

old_synth = "    // Dimenzióváltó primitívek induktív kipróbálása."
new_synth = "    // MetaGrid-alapú BlockMap generálás\n    if let Some(meta) = athlesia_structure::TargetDecomposer::decompose_dimensions(input, target) {\n        // A MetaGrid celláiból készítünk BlockMap paramétereket\n        let mut transforms: Vec<u8> = Vec::new();\n        // A decompose-ot használjuk a transzformációk felismerésére\n        let decomposer = athlesia_structure::TargetDecomposer;\n        if let Some(grid_meta) = decomposer.decompose(input, target) {\n            for cell in &grid_meta.cells {\n                transforms.push(match cell {\n                    Some(athlesia_structure::TransformId::Identity) => 0,\n                    Some(athlesia_structure::TransformId::Rot90) => 1,\n                    Some(athlesia_structure::TransformId::Rot180) => 2,\n                    Some(athlesia_structure::TransformId::Rot270) => 3,\n                    Some(athlesia_structure::TransformId::ReflectH) => 4,\n                    Some(athlesia_structure::TransformId::ReflectV) => 5,\n                    None => return None,\n                });\n            }\n            let program = vec![(\n                PrimName::BlockMap,\n                Params::BlockMap(meta.block_rows, meta.block_cols, transforms),\n            )];\n            let mut budget = Budget { max_steps: 1, max_depth: 100 };\n            if let Ok(output) = run_program(&program, input, &mut budget) {\n                if output == *target {\n                    return Some(program);\n                }\n            }\n        }\n    }\n\n    // Dimenzióváltó primitívek induktív kipróbálása."

if old_synth in s:
    s = s.replace(old_synth, new_synth)
    write_file(p, s)
    print("[3] Synthesis Engine MetaGrid-alapú BlockMap generálással bővítve.")
else:
    print("[ERROR] Nem találtam a dimenzióváltó szintézis blokkot.")
    sys.exit(1)

# 4. Tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-synthesis", "--test", "dimension_test"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Phase 3 szintézis tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Phase 3 szintézis tesztek zöldek.")

# 5. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Add MetaGrid-based BlockMap synthesis"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
