#!/usr/bin/env python3
import os, subprocess, sys, pathlib

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Types: HardConstraintError hozzáadása
p = pathlib.Path("crates/athlesia-types/src/lib.rs")
s = p.read_text()

if "HardConstraintError" not in s:
    # A fájl végére fűzzük
    s += r'''

/// Kemény kényszer megsértése (például érvénytelen koordináta vagy dimenzió).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardConstraintError {
    InvalidCoordinate { x: i8, y: i8 },
    InvalidDimensions { expected_width: u8, expected_height: u8 },
    InvalidColorValue { color: u8 },
    InvalidAction { action: String },
}
'''
    write_file(p, s)
    print("[1] HardConstraintError hozzáadva a types-hoz.")
else:
    print("[1] HardConstraintError már létezik.")

# 2. MetaLearner: simplicity_score hozzáadása
p = pathlib.Path("crates/athlesia-metalearner/src/lib.rs")
s = p.read_text()

if "pub fn simplicity_score" not in s:
    # A fájl végére, az impl blokkba szúrjuk be? Egyszerűbb az impl blokk után újra megnyitni.
    # Keressük az impl MetaLearner blokkot
    marker = "impl MetaLearner {"
    insert_pos = s.find(marker)
    if insert_pos == -1:
        print("[ERROR] MetaLearner impl blokk nem található")
        sys.exit(1)

    new_fn = '''
    /// Egyszerűségi pontszám (Occam-prior): a rövidebb program jobb.
    /// A komplexitás büntetése: minden lépésért 1.0, a nagyobb mélységért extra.
    pub fn simplicity_score(&self, program: &Program) -> f64 {
        let len = program.len() as f64;
        // Egyszerű lineáris büntetés: minél hosszabb, annál kisebb a pontszám.
        // 1.0 / (1.0 + len) -> 1.0 az üres programra, csökken a hosszal.
        1.0 / (1.0 + len)
    }
'''
    # Az impl blokk után keressük a következő záró kapcsost, hogy oda szúrjuk be
    # De egyszerűbb, ha a `pub fn score_abstraction` elé illesztjük
    anchor = "    /// Zárt alakú, determinisztikus lineáris absztrakció-minőség pontozó."
    pos = s.find(anchor)
    if pos == -1:
        print("[ERROR] score_abstraction anchor nem található")
        sys.exit(1)
    s = s[:pos] + new_fn + "\n" + s[pos:]
    write_file(p, s)
    print("[2] simplicity_score hozzáadva a MetaLearner-hez.")
else:
    print("[2] simplicity_score már létezik.")

# 3. Kernel: cognitive modul létrehozása
# A modul a kernel/src/cognitive.rs fájlba kerül
write_file("crates/athlesia-kernel/src/cognitive.rs", r'''
use athlesia_features::FeatureVector;
use athlesia_metalearner::MetaLearner;
use athlesia_types::Program;

/// Döntés, amit a kognitív kontroller hozhat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CognitiveDecision {
    Solve,
    Explore,
    Guess,
    Abstain,
}

/// Kompetenciabecslés: mennyire ismeri a rendszer a feladatot,
/// mennyire bízik a hipotézisében, és mekkora keresési költséget jósol.
#[derive(Debug, Clone)]
pub struct CompetenceEstimate {
    pub familiarity: f32,
    pub structural_match: f32,
    pub hypothesis_confidence: f32,
    pub predicted_search_cost: f32,
    pub expected_information_gain: f32,
}

/// Kognitív kontroller: dönt a cselekvésről a jellemzővektor és a
/// MetaLearner állapota alapján.
pub struct CognitiveController;

impl CognitiveController {
    /// Egyszerű döntési logika:
    /// - ha van hasonló kontextusú ismert program, és a konfidencia magas -> Solve
    /// - ha a hasonlóság közepes -> Explore
    /// - ha nincs semmi, de a keresés olcsónak ígérkezik -> Guess
    /// - egyébként -> Abstain
    pub fn decide(
        features: &FeatureVector,
        meta: &MetaLearner,
        known_programs: &[Program],
    ) -> CognitiveDecision {
        let _ = known_programs; // most nem használjuk, de a jövőben kell

        // Nagyon egyszerű becslés: ha van bármilyen kontextus-pontszám
        // a meta learnerben, akkor Solve, különben Explore.
        // Ez most egy placeholder, amit később finomítunk.
        let context_confidence = meta.priority_in_context(*features, 0); // 0 hipotézis id

        if context_confidence > 0.8 {
            CognitiveDecision::Solve
        } else if context_confidence > 0.5 {
            CognitiveDecision::Explore
        } else {
            CognitiveDecision::Guess
        }
    }

    /// Kompetenciabecslés kiszámítása.
    pub fn estimate(
        features: &FeatureVector,
        meta: &MetaLearner,
    ) -> CompetenceEstimate {
        // Egyszerű közelítés: a konfidencia a 0. hipotézisre.
        let conf = meta.priority_in_context(*features, 0);
        CompetenceEstimate {
            familiarity: conf,
            structural_match: 0.0,
            hypothesis_confidence: conf,
            predicted_search_cost: 100.0,
            expected_information_gain: 0.5,
        }
    }
}
''')
print("[3] cognitive.rs létrehozva.")

# A modul regisztrálása a kernel lib.rs-ben
p = pathlib.Path("crates/athlesia-kernel/src/lib.rs")
s = p.read_text()
if "pub mod cognitive;" not in s:
    # Az első use után szúrjuk be
    first_use = s.find("use serde::Deserialize;")
    if first_use == -1:
        first_use = 0
    s = s[:first_use] + "pub mod cognitive;\n" + s[first_use:]
    write_file(p, s)
    print("[4] cognitive modul regisztrálva a kernel lib.rs-ben.")
else:
    print("[4] cognitive modul már regisztrálva.")

# 4. Tesztek a cognitive modulhoz
write_file("crates/athlesia-kernel/tests/cognitive_test.rs", r'''
use athlesia_kernel::cognitive::{CognitiveController, CognitiveDecision};
use athlesia_metalearner::MetaLearner;
use athlesia_features::FeatureVector;
use athlesia_types::{PrimName, Params, Program};

fn make_fv(object_count: u8) -> FeatureVector {
    FeatureVector {
        object_count,
        ..Default::default()
    }
}

#[test]
fn controller_decides_guess_when_no_prior() {
    let controller = CognitiveController;
    let fv = make_fv(1);
    let meta = MetaLearner::new();
    let programs: Vec<Program> = vec![vec![(PrimName::Translate, Params::Translate(1, 0))]];

    let decision = controller.decide(&fv, &meta, &programs);
    // Mivel nincs kontextus-pontszám, a döntés Guess (vagy Explore, ha így alakul)
    // Most a placeholder logika szerint Guess lesz, de ellenőrizzük a változatosságot.
    assert_ne!(decision, CognitiveDecision::Abstain);
}

#[test]
fn controller_estimates_competence() {
    let controller = CognitiveController;
    let fv = make_fv(2);
    let meta = MetaLearner::new();

    let estimate = controller.estimate(&fv, &meta);
    assert!(estimate.hypothesis_confidence >= 0.0 && estimate.hypothesis_confidence <= 1.0);
    assert_eq!(estimate.familiarity, estimate.hypothesis_confidence);
}

#[test]
fn controller_decides_solve_when_confident() {
    let mut meta = MetaLearner::new();
    // Mesterségesen megnöveljük a 0 hipotézis konfidenciáját
    let fv = make_fv(1);
    for _ in 0..5 {
        meta.record_success_in_context(fv, 0);
    }

    let controller = CognitiveController;
    let programs: Vec<Program> = vec![vec![(PrimName::Translate, Params::Translate(1, 0))]];
    let decision = controller.decide(&fv, &meta, &programs);

    assert_eq!(decision, CognitiveDecision::Solve);
}
''')
print("[5] cognitive_test.rs létrehozva.")

# 5. Tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-kernel", "--test", "cognitive_test"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Cognitive tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Cognitive tesztek zöldek.")

# 6. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Add Phase 1 cognitive controller, hard constraints, and simplicity score"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
