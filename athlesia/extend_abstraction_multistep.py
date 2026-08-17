#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
ABS_DIR = os.path.join(PROJECT, "crates", "athlesia-abstraction")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Abstraction lib.rs frissítése: 1 és 2 hosszú minták kinyerése
lib_path = os.path.join(ABS_DIR, "src", "lib.rs")
content = pathlib.Path(lib_path).read_text()

# Régi extract_macros cseréje
old_fn = '''    pub fn extract_macros(
        solved_programs: &[Program],
        kb: &mut KnowledgeBase,
        threshold: usize,
    ) -> usize {
        let mut counts: HashMap<Program, usize> = HashMap::new();

        // Számláljuk az egylépéses programokat
        for program in solved_programs {
            if program.len() == 1 {
                *counts.entry(program.clone()).or_insert(0) += 1;
            }
        }

        let mut added = 0;
        for (program, count) in counts {
            if count >= threshold {
                // Ellenőrizzük, hogy nincs-e már ilyen nevű makró
                let name = format!("macro_{}", kb.get_all_macros().len());
                let exists = kb.get_all_macros().iter().any(|m| m.program == program);
                if !exists {
                    kb.add_macro(name, program);
                    added += 1;
                }
            }
        }

        added
    }'''

new_fn = '''    pub fn extract_macros(
        solved_programs: &[Program],
        kb: &mut KnowledgeBase,
        threshold: usize,
    ) -> usize {
        let mut counts: HashMap<Program, usize> = HashMap::new();

        for program in solved_programs {
            // Egylépéses minták
            for step in program {
                let sub = vec![step.clone()];
                *counts.entry(sub).or_insert(0) += 1;
            }

            // Kétlépéses összefüggő minták
            if program.len() >= 2 {
                for window in program.windows(2) {
                    let sub: Program = window.to_vec();
                    *counts.entry(sub).or_insert(0) += 1;
                }
            }
        }

        let mut added = 0;
        for (pattern, count) in counts {
            if count >= threshold {
                // Ellenőrizzük, hogy nincs-e már ilyen program a könyvtárban
                let exists = kb.get_all_macros().iter().any(|m| m.program == pattern);
                if !exists {
                    let name = format!("macro_{}", kb.get_all_macros().len());
                    kb.add_macro(name, pattern);
                    added += 1;
                }
            }
        }

        added
    }'''

if old_fn in content:
    content = content.replace(old_fn, new_fn)
    write_file(lib_path, content)
    print("[INFO] extract_macros bővítve 1 és 2 lépéses mintákra.")
else:
    print("[ERROR] Nem találtam a régi extract_macros függvényt.")
    sys.exit(1)

# 2. Teszt hozzáadása: kétlépéses minta kinyerése
test_content = r'''
use athlesia_abstraction::AbstractionEngine;
use athlesia_knowledge::KnowledgeBase;
use athlesia_types::{PrimName, Params, Program};

#[test]
fn extracts_frequent_two_step_macro() {
    let mut kb = KnowledgeBase::new();
    let engine = AbstractionEngine;

    let translate: (PrimName, Params) = (PrimName::Translate, Params::Translate(1, 0));
    let recolor: (PrimName, Params) = (PrimName::Recolor, Params::Recolor([1, 0, 2, 3]));

    // A gyakori kétlépéses minta: translate + recolor
    let pattern: Program = vec![translate, recolor];

    // Hét megoldott program, mindegyik tartalmazza ezt a mintát,
    // de különböző további lépésekkel.
    let solved = vec![
        pattern.clone(),
        vec![translate, recolor, (PrimName::ReflectH, Params::None)],
        vec![(PrimName::Rotate90, Params::None), translate, recolor],
        pattern.clone(),
        vec![translate, recolor, (PrimName::ReflectV, Params::None)],
        pattern.clone(),
        vec![(PrimName::Translate, Params::Translate(0, 1)), translate, recolor],
    ];

    let added = engine.extract_macros(&solved, &mut kb, 5);
    assert!(added >= 1, "Legalább egy makrót hozzá kell adni, hozzáadva: {}", added);

    // A kétlépéses mintának szerepelnie kell a makrók között
    let has_pattern = kb.get_all_macros().iter().any(|m| m.program == pattern);
    assert!(has_pattern, "A gyakori kétlépéses mintát makróként kell tárolni");
}

#[test]
fn does_not_extract_infrequent_two_step_macro() {
    let mut kb = KnowledgeBase::new();
    let engine = AbstractionEngine;

    let a: Program = vec![
        (PrimName::Translate, Params::Translate(1, 0)),
        (PrimName::Recolor, Params::Recolor([1, 0, 2, 3])),
    ];
    let b: Program = vec![
        (PrimName::ReflectH, Params::None),
        (PrimName::Rotate90, Params::None),
    ];

    let solved = vec![a.clone(), b.clone()];

    let added = engine.extract_macros(&solved, &mut kb, 2);
    assert_eq!(added, 0, "Nem szabad makrót hozzáadni, mert nincs elég gyakori minta");
}
'''
write_file(os.path.join(ABS_DIR, "tests", "multistep_abstraction_test.rs"), test_content)
print("[INFO] Többlépéses absztrakció teszt hozzáadva.")

# 3. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-abstraction"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Abstraction multistep tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Abstraction multistep tesztek zöldek.")

# 4. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Extend abstraction engine to multi-step patterns"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
