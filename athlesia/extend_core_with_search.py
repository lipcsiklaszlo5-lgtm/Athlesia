#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
CORE_DIR = os.path.join(PROJECT, "crates", "athlesia-core")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Core Cargo.toml-hez a search függőség hozzáadása
cargo_path = os.path.join(CORE_DIR, "Cargo.toml")
cargo_content = pathlib.Path(cargo_path).read_text()
if "athlesia-search" not in cargo_content:
    cargo_content = cargo_content.replace(
        "[dependencies]\n",
        "[dependencies]\nathlesia-search = { path = \"../athlesia-search\" }\n"
    )
    write_file(cargo_path, cargo_content)
    print("[INFO] search függőség hozzáadva a Core-hoz.")

# 2. Core lib.rs bővítése a search hívásával
lib_path = os.path.join(CORE_DIR, "src", "lib.rs")
lib_content = pathlib.Path(lib_path).read_text()

# Import bővítése
lib_content = lib_content.replace(
    "use athlesia_synthesis::{synthesize, PrimitiveTemplate};",
    "use athlesia_synthesis::{synthesize, PrimitiveTemplate};\nuse athlesia_search::search;"
)

# A szintézis utáni ág kiegészítése
old_snippet = '''        // 2. Ha nincs megfelelő, szintetizáljunk
        let templates = vec![
            PrimitiveTemplate::Translate,
            PrimitiveTemplate::ReflectH,
            PrimitiveTemplate::ReflectV,
            PrimitiveTemplate::Rotate90,
            PrimitiveTemplate::Recolor,
        ];

        if let Some(program) = synthesize(input, target, &templates) {
            steps += 1; // a szintézis egy próbálkozásnak számít
            // Verifikáljuk a szintetizált programot
            if self.verifier.verify(&program, &[(input.clone(), target.clone())]) == VerificationResult::Accept {
                let id = self.known_programs.len() as u64;
                self.known_programs.push(program.clone());
                self.meta.record_success_in_context(fv, id);
                return (Some(program), steps);
            }
        }

        (None, steps)
    }'''

new_snippet = '''        // 2. Ha nincs megfelelő, szintetizáljunk
        let templates = vec![
            PrimitiveTemplate::Translate,
            PrimitiveTemplate::ReflectH,
            PrimitiveTemplate::ReflectV,
            PrimitiveTemplate::Rotate90,
            PrimitiveTemplate::Recolor,
        ];

        if let Some(program) = synthesize(input, target, &templates) {
            steps += 1; // a szintézis egy próbálkozásnak számít
            // Verifikáljuk a szintetizált programot
            if self.verifier.verify(&program, &[(input.clone(), target.clone())]) == VerificationResult::Accept {
                let id = self.known_programs.len() as u64;
                self.known_programs.push(program.clone());
                self.meta.record_success_in_context(fv, id);
                return (Some(program), steps);
            }
        }

        // 3. Ha a szintézis nem járt sikerrel, próbáljuk a többlépéses keresést
        if let Some(program) = search(input, target, 3) {
            steps += 1;
            if self.verifier.verify(&program, &[(input.clone(), target.clone())]) == VerificationResult::Accept {
                let id = self.known_programs.len() as u64;
                self.known_programs.push(program.clone());
                self.meta.record_success_in_context(fv, id);
                return (Some(program), steps);
            }
        }

        (None, steps)
    }'''

if old_snippet in lib_content:
    lib_content = lib_content.replace(old_snippet, new_snippet)
    write_file(lib_path, lib_content)
    print("[INFO] CoreEngine bővítve többlépéses kereséssel.")
else:
    print("[ERROR] Nem találtam a cserélendő kódrészletet.")
    sys.exit(1)

# 3. Új teszt: kétlépéses megoldás megtalálása
test_content = r'''
use athlesia_core::CoreEngine;
use athlesia_types::{Grid, PrimName, Params};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn solves_two_step_program_with_search() {
    let mut core = CoreEngine::new();

    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    // Cél: két lépés jobbra (Translate(1,0) + Translate(1,0)) = Translate(2,0)
    let target = build_grid([
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let (program, steps) = core.solve_with_steps(&input, &target);
    assert!(program.is_some(), "A motornak meg kell oldania a kétlépéses feladatot");
    // A megoldás hossza legalább 2, mert két Translate lépés kell
    let program = program.unwrap();
    assert!(program.len() >= 2, "A programnak legalább 2 lépésből kell állnia, de hossza: {}", program.len());

    // A megtanult programot a motor másodjára már azonnal előveszi
    let (_, steps_second) = core.solve_with_steps(&input, &target);
    assert_eq!(steps_second, 1, "Másodjára már csak 1 lépés kell");
}
'''
write_file(os.path.join(CORE_DIR, "tests", "search_learning_test.rs"), test_content)
print("[INFO] Kétlépéses keresés teszt hozzáadva.")

# 4. Tesztek futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-core"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Core search tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Core search tesztek zöldek.")

# 5. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Enable multi-step search in CoreEngine"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
