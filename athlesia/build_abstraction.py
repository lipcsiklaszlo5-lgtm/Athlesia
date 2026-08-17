#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
ABS_DIR = os.path.join(PROJECT, "crates", "athlesia-abstraction")
WORKSPACE_TOML = os.path.join(PROJECT, "Cargo.toml")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Workspace frissítése
ws = pathlib.Path(WORKSPACE_TOML).read_text()
if "athlesia-abstraction" not in ws:
    ws = ws.replace(
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner", "crates/athlesia-verifier", "crates/athlesia-synthesis", "crates/athlesia-core", "crates/athlesia-search", "crates/athlesia-memory", "crates/athlesia-knowledge"]',
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner", "crates/athlesia-verifier", "crates/athlesia-synthesis", "crates/athlesia-core", "crates/athlesia-search", "crates/athlesia-memory", "crates/athlesia-knowledge", "crates/athlesia-abstraction"]'
    )
    write_file(WORKSPACE_TOML, ws)
    print("[INFO] Workspace frissítve.")

# 2. Abstraction crate létrehozása
os.makedirs(os.path.join(ABS_DIR, "src"), exist_ok=True)
os.makedirs(os.path.join(ABS_DIR, "tests"), exist_ok=True)

write_file(os.path.join(ABS_DIR, "Cargo.toml"), '''[package]
name = "athlesia-abstraction"
version = "0.1.0"
edition = "2021"

[dependencies]
athlesia-types = { path = "../athlesia-types" }
athlesia-knowledge = { path = "../athlesia-knowledge" }
''')

write_file(os.path.join(ABS_DIR, "src", "lib.rs"), r'''
use std::collections::HashMap;
use athlesia_types::Program;
use athlesia_knowledge::KnowledgeBase;

/// Abstraction Engine: gyakori programminták kinyerése és makrósítása.
///
/// Az aktuális implementáció a legegyszerűbb, de valódi absztrakció:
/// megkeresi azokat az egylépéses programokat (primitív + paraméter),
/// amelyek legalább `threshold` alkalommal előfordulnak a megoldott
/// programok között, és amelyek még nincsenek makróként a tudásbázisban.
/// A megtalált mintákat makróként hozzáadja a tudásbázishoz.
///
/// A későbbi fázisokban ez bővül majd anti-unifikációval és MDL-pontozással.
pub struct AbstractionEngine;

impl AbstractionEngine {
    /// Megoldott programokból makrókat emel ki.
    /// `solved_programs`: a megoldott feladatok programjai.
    /// `kb`: a tudásbázis, amibe az új makrók kerülnek.
    /// `threshold`: hány előfordulás felett tekintünk egy mintát érdemesnek.
    pub fn extract_macros(
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
    }
}
''')

# 3. Tesztek
write_file(os.path.join(ABS_DIR, "tests", "abstraction_test.rs"), r'''
use athlesia_abstraction::AbstractionEngine;
use athlesia_knowledge::KnowledgeBase;
use athlesia_types::{PrimName, Params, Program};

#[test]
fn extracts_frequent_single_step_program() {
    let mut kb = KnowledgeBase::new();
    let engine = AbstractionEngine;

    let translate: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];
    let reflect: Program = vec![(PrimName::ReflectH, Params::None)];

    // Négy megoldott program, ebből három ugyanaz a translate
    let solved = vec![
        translate.clone(),
        translate.clone(),
        translate.clone(),
        reflect.clone(),
    ];

    let added = engine.extract_macros(&solved, &mut kb, 3);
    assert_eq!(added, 1, "Egy makrót kell hozzáadni");
    assert_eq!(kb.get_all_macros().len(), 1);
    assert_eq!(kb.get_all_macros()[0].program, translate);
}

#[test]
fn does_not_extract_infrequent_program() {
    let mut kb = KnowledgeBase::new();
    let engine = AbstractionEngine;

    let translate: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];
    let reflect: Program = vec![(PrimName::ReflectH, Params::None)];

    let solved = vec![translate.clone(), reflect.clone()];

    let added = engine.extract_macros(&solved, &mut kb, 2);
    assert_eq!(added, 0, "Nem szabad makrót hozzáadni, mert nincs elég gyakori minta");
    assert_eq!(kb.get_all_macros().len(), 0);
}

#[test]
fn respects_existing_macro() {
    let mut kb = KnowledgeBase::new();
    let engine = AbstractionEngine;

    let translate: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];

    // Már létező makró
    kb.add_macro("existing_macro".to_string(), translate.clone());

    let solved = vec![translate.clone(), translate.clone(), translate.clone()];

    let added = engine.extract_macros(&solved, &mut kb, 2);
    assert_eq!(added, 0, "Nem szabad duplikátum makrót hozzáadni");
    assert_eq!(kb.get_all_macros().len(), 1);
}
''')

print("[INFO] Abstraction crate létrehozva.")

# 4. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-abstraction"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Abstraction tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Abstraction tesztek zöldek.")

# 5. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add athlesia-abstraction module"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
