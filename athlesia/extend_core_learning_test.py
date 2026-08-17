#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
CORE_DIR = os.path.join(PROJECT, "crates", "athlesia-core")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. CoreEngine lib.rs bővítése solve_with_steps metódussal
lib_path = os.path.join(CORE_DIR, "src", "lib.rs")
lib_content = pathlib.Path(lib_path).read_text()

if "pub fn solve_with_steps" not in lib_content:
    # Meglévő solve metódus lecserélése solve_with_steps-re
    old_solve = '''
    pub fn solve(&mut self, input: &Grid, target: &Grid) -> Option<Program> {
        let fv = extract_features(input);
        let ids: Vec<u64> = (0..self.known_programs.len() as u64).collect();

        // 1. Próbáljuk a már ismert programokat a MetaLearner rangsora szerint
        let ranked = self.meta.rank_in_context(fv, &ids);
        for id in ranked {
            let program = self.known_programs[id as usize].clone();
            let result = self.verifier.verify(&program, &[(input.clone(), target.clone())]);
            if result == VerificationResult::Accept {
                self.meta.record_success_in_context(fv, id);
                return Some(program);
            } else {
                self.meta.record_failure_in_context(fv, id);
            }
        }

        // 2. Ha nincs megfelelő, szintetizáljunk
        let templates = vec![
            PrimitiveTemplate::Translate,
            PrimitiveTemplate::ReflectH,
            PrimitiveTemplate::ReflectV,
            PrimitiveTemplate::Rotate90,
            PrimitiveTemplate::Recolor,
        ];

        if let Some(program) = synthesize(input, target, &templates) {
            // Verifikáljuk a szintetizált programot
            if self.verifier.verify(&program, &[(input.clone(), target.clone())]) == VerificationResult::Accept {
                let id = self.known_programs.len() as u64;
                self.known_programs.push(program.clone());
                self.meta.record_success_in_context(fv, id);
                return Some(program);
            }
        }

        None
    }
'''
    new_solve = '''
    /// Megoldja a feladatot, és visszaadja a megtalált programot.
    pub fn solve(&mut self, input: &Grid, target: &Grid) -> Option<Program> {
        self.solve_with_steps(input, target).0
    }

    /// Megoldja a feladatot, és visszaadja a megtalált programot,
    /// valamint azt, hogy hány hipotézist próbált ki (keresési lépések).
    pub fn solve_with_steps(&mut self, input: &Grid, target: &Grid) -> (Option<Program>, usize) {
        let fv = extract_features(input);
        let ids: Vec<u64> = (0..self.known_programs.len() as u64).collect();

        let mut steps = 0;

        // 1. Próbáljuk a már ismert programokat a MetaLearner rangsora szerint
        let ranked = self.meta.rank_in_context(fv, &ids);
        for id in ranked {
            steps += 1;
            let program = self.known_programs[id as usize].clone();
            let result = self.verifier.verify(&program, &[(input.clone(), target.clone())]);
            if result == VerificationResult::Accept {
                self.meta.record_success_in_context(fv, id);
                return (Some(program), steps);
            } else {
                self.meta.record_failure_in_context(fv, id);
            }
        }

        // 2. Ha nincs megfelelő, szintetizáljunk
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
    }
'''
    if old_solve in lib_content:
        lib_content = lib_content.replace(old_solve, new_solve)
        write_file(lib_path, lib_content)
        print("[INFO] solve_with_steps hozzáadva a CoreEngine-hez.")
    else:
        print("[ERROR] Nem találtam a régi solve metódust.")
        sys.exit(1)
else:
    print("[INFO] solve_with_steps már létezik.")

# 2. Tanulási görbe teszt hozzáadása
test_content = r'''
use athlesia_core::CoreEngine;
use athlesia_types::{Grid, PrimName, Params};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn search_steps_decrease_after_learning() {
    let mut core = CoreEngine::new();

    // Néhány félrevezető, rossz program, hogy az első feladatnál sok hipotézist bukjon el
    core.known_programs.push(vec![(PrimName::ReflectH, Params::None)]);
    core.known_programs.push(vec![(PrimName::ReflectV, Params::None)]);
    core.known_programs.push(vec![(PrimName::Rotate90, Params::None)]);
    core.known_programs.push(vec![(PrimName::Recolor, Params::Recolor([1, 0, 2, 3]))]);

    // Ugyanaz a szabály: jobbra tolás (Translate(1,0))
    // Különböző kiinduló cellák, de a jellemzővektoruk azonos lesz,
    // mert egyetlen 1-es színű cellából állnak.
    let make_input = |pos: (usize, usize)| -> Grid {
        let mut rows = [[0u8; 5]; 5];
        rows[pos.0][pos.1] = 1;
        build_grid(rows)
    };

    let mut steps_history = Vec::new();

    for i in 0..5 {
        let (x, y) = (i % 3, i); // változó pozíciók, de ugyanaz a szabály
        let input = make_input((x, y));
        let mut target_rows = [[0u8; 5]; 5];
        target_rows[x][(y + 1).min(4)] = 1; // jobbra tolás (ha fér)
        let target = build_grid(target_rows);

        let (program, steps) = core.solve_with_steps(&input, &target);
        assert!(program.is_some(), "A kernelnek meg kell oldania a feladatot");
        steps_history.push(steps);
    }

    // Az első feladatnál legalább 4 rossz programot bukik el + szintézis
    assert!(steps_history[0] >= 5, "Első lépésszám: {}", steps_history[0]);
    // A második feladattól a megtanult program már az első helyen van
    assert_eq!(steps_history[1], 1, "Második lépésszám: {}", steps_history[1]);
    assert_eq!(steps_history[4], 1, "Utolsó lépésszám: {}", steps_history[4]);

    // A keresési lépésszám csökkenése
    let avg_first_two = (steps_history[0] + steps_history[1]) / 2;
    let avg_last_two = (steps_history[3] + steps_history[4]) / 2;
    assert!(avg_last_two < avg_first_two, "Tanulási görbe nem csökkent: {:?}", steps_history);
}
'''
write_file(os.path.join(CORE_DIR, "tests", "learning_curve_test.rs"), test_content)
print("[INFO] Tanulási görbe teszt hozzáadva.")

# 3. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-core"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Core tanulási teszt nem ment át.")
    sys.exit(1)
print("\n[SUCCESS] Core tanulási teszt zöld.")

# 4. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add learning curve test with step counting"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
