#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Core Cargo.toml bővítése szükséges függőségekkel
p = pathlib.Path("crates/athlesia-core/Cargo.toml")
s = p.read_text()

deps_to_add = [
    "athlesia-world-model = { path = \"../athlesia-world-model\" }",
    "athlesia-hypothesis = { path = \"../athlesia-hypothesis\" }",
    "athlesia-abstraction = { path = \"../athlesia-abstraction\" }",
    "athlesia-knowledge = { path = \"../athlesia-knowledge\" }",
]
if "[dependencies]" in s:
    # ellenőrizzük, melyek hiányoznak
    for dep in deps_to_add:
        crate_name = dep.split("=")[0].strip()
        if crate_name not in s:
            s = s.replace("[dependencies]", "[dependencies]\n" + dep, 1)
else:
    deps_block = "\n[dependencies]\n" + "\n".join(deps_to_add) + "\n"
    s += deps_block

p.write_text(s)
print("[1] core Cargo.toml frissítve.")

# 2. Új openworld modul létrehozása
openworld_code = r'''
use athlesia_world_model::{WorldModel, KnowledgeState, Prediction, Observation, Action, PredictionResidual};
use athlesia_abstraction::AbstractionEngine;
use athlesia_hypothesis::{CandidateConcept, ConceptSketch};
use athlesia_knowledge::KnowledgeBase;

/// Open-world ciklus: reziduálisból fogalomjelölt, majd egyszerű verifikáció.
pub struct OpenWorldCycle;

impl OpenWorldCycle {
    /// Lefuttatja a Phase 13 alapciklust.
    ///
    /// 1. Kiértékeli a predikciót (tudásállapot + reziduális).
    /// 2. Ha OutOfModel, a reziduálisból candidate conceptet generál.
    /// 3. Ha a candidate confidence elég magas, verifikálja és beilleszti a KB-be.
    pub fn run(
        wm: &WorldModel,
        action: &Action,
        prediction: &Prediction,
        observation: &Observation,
        kb: &mut KnowledgeBase,
    ) -> Option<athlesia_knowledge::VerifiedConcept> {
        let (state, residual) = wm.evaluate_with_residual(action, prediction, observation);
        if state != KnowledgeState::OutOfModel {
            return None;
        }

        let residuals = vec![residual];
        let candidate = AbstractionEngine::discover_candidate_concept(&residuals)?;

        // Egyszerű verifikáció: ha a candidate confidence elér egy küszöböt,
        // igazolt fogalomként kezeljük.
        if candidate.confidence >= 0.5 {
            let verified = athlesia_knowledge::VerifiedConcept {
                id: kb.get_verified_concepts().len() as u64,
                name: candidate.sketch.name.clone(),
                relation_pattern: candidate.sketch.relation_pattern.clone(),
                evidence_count: candidate.evidence.len(),
            };
            kb.add_verified_concept(
                verified.name.clone(),
                verified.relation_pattern.clone(),
                verified.evidence_count,
            );
            Some(verified)
        } else {
            None
        }
    }
}
'''

write_file("crates/athlesia-core/src/openworld.rs", openworld_code)
print("[2] openworld.rs létrehozva.")

# 3. Modul deklaráció hozzáadása a core lib.rs-hez
p = pathlib.Path("crates/athlesia-core/src/lib.rs")
s = p.read_text()
if "pub mod openworld;" not in s:
    # a meglévő use-ok után, de a struct előtt szúrjuk be
    s = s.replace("use athlesia_types::{Grid, Program};", "use athlesia_types::{Grid, Program};\n\npub mod openworld;", 1)
    p.write_text(s)
    print("[3] core lib.rs: mod openworld hozzáadva.")
else:
    print("[3] core lib.rs már tartalmazza a modult.")

# 4. Tesztfájl
test_code = r'''
use athlesia_core::openworld::OpenWorldCycle;
use athlesia_world_model::{WorldModel, Observation, Prediction};
use athlesia_types::{Grid, Color, Action, PrimName, Params};
use athlesia_knowledge::KnowledgeBase;

fn grid_5x5_with_pixel(x: usize, y: usize, val: u8) -> Grid {
    let mut cells = vec![Color(0); 25];
    cells[y * 5 + x] = Color(val);
    Grid { width: 5, height: 5, cells }
}

#[test]
fn openworld_cycle_creates_verified_concept_on_out_of_model() {
    let initial = grid_5x5_with_pixel(0, 0, 1);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    // WorldModel csak ReflectH hipotézissel
    let mut wm = WorldModel::new(initial.clone());
    wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);

    let prediction = wm.predict(&initial, &action);
    // Megfigyelés rossz, hogy OutOfModel legyen
    let observation = Observation { state: initial.clone() };

    let mut kb = KnowledgeBase::new();
    let verified = OpenWorldCycle::run(&wm, &action, &prediction, &observation, &mut kb);

    assert!(verified.is_some(), "A ciklusnak igazolt fogalmat kell létrehoznia");
    assert_eq!(kb.get_verified_concepts().len(), 1);
}

#[test]
fn openworld_cycle_no_verified_concept_when_not_out_of_model() {
    let initial = grid_5x5_with_pixel(0, 0, 1);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let wm = WorldModel::new(initial.clone()); // nincs hipotézis

    let prediction = wm.predict(&initial, &action);
    let observation = Observation { state: initial.clone() };

    let mut kb = KnowledgeBase::new();
    let verified = OpenWorldCycle::run(&wm, &action, &prediction, &observation, &mut kb);

    assert!(verified.is_none(), "Nem szabad fogalmat létrehozni, ha nem OutOfModel");
}
'''

write_file("crates/athlesia-core/tests/openworld_cycle_test.rs", test_code)
print("[4] openworld_cycle_test.rs létrehozva.")

# 5. Core tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-core"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Core tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Core tesztek zöldek.")

# 6. Teljes workspace teszt
result = subprocess.run(
    ["cargo", "test", "--workspace", "--no-fail-fast"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Teljes workspace tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Teljes workspace tesztek zöldek.")

# 7. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 8: open-world cycle in CoreEngine creates verified concepts from residuals"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
