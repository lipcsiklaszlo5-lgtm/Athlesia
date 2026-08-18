#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. openworld.rs bővítése OpenWorldOutcome enummal és run_with_outcome metódussal
p = pathlib.Path("crates/athlesia-core/src/openworld.rs")
s = p.read_text()

# Enum beszúrása az OpenWorldCycle struct után
anchor = "pub struct OpenWorldCycle;"
new_enum = anchor + '''

/// Az open-world ciklus kimenete.
#[derive(Debug, Clone, PartialEq)]
pub enum OpenWorldOutcome {
    NotOutOfModel,
    Abstain,
    Retrieved(athlesia_knowledge::VerifiedConcept),
    Verified(athlesia_knowledge::VerifiedConcept),
}
'''
if anchor not in s:
    print("[ERROR] OpenWorldCycle struct nem található.")
    sys.exit(1)
s = s.replace(anchor, new_enum)

# run_with_outcome metódus beszúrása az impl blokk elejére (a run elé)
impl_anchor = "impl OpenWorldCycle {"
new_impl_start = impl_anchor + '''

    /// Az open-world ciklus kimenettel együtt.
    ///
    /// - Ha nem OutOfModel: NotOutOfModel
    /// - Ha van OutOfModel, de a candidate confidence < 0.5: Abstain
    /// - Ha a relation_pattern már létezik: Retrieved
    /// - Különben Verified
    pub fn run_with_outcome(
        wm: &WorldModel,
        action: &Action,
        prediction: &Prediction,
        observation: &Observation,
        kb: &mut KnowledgeBase,
    ) -> OpenWorldOutcome {
        let (state, residual) = wm.evaluate_with_residual(action, prediction, observation);
        if state != KnowledgeState::OutOfModel {
            return OpenWorldOutcome::NotOutOfModel;
        }

        let residuals = vec![residual];
        let candidate = match AbstractionEngine::discover_candidate_concept(&residuals) {
            Some(c) => c,
            None => return OpenWorldOutcome::Abstain,
        };

        if let Some(existing) = kb
            .get_verified_concepts()
            .iter()
            .find(|c| c.relation_pattern == candidate.sketch.relation_pattern)
        {
            return OpenWorldOutcome::Retrieved(existing.clone());
        }

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
            OpenWorldOutcome::Verified(verified)
        } else {
            OpenWorldOutcome::Abstain
        }
    }
'''
if impl_anchor not in s:
    print("[ERROR] impl OpenWorldCycle nem található.")
    sys.exit(1)
s = s.replace(impl_anchor, new_impl_start)

# A régi run metódus is megmarad, de áthívjuk az újra? Megtartjuk a kompatibilitás miatt.
# A run jelenleg a régi logikát tartalmazza; nem módosítjuk, mert a tesztek azt várják.

p.write_text(s)
print("[1] openworld.rs bővítve: OpenWorldOutcome és run_with_outcome.")

# 2. Új tesztfájl
test_code = r'''
use athlesia_core::openworld::{OpenWorldCycle, OpenWorldOutcome};
use athlesia_world_model::{WorldModel, Observation, Prediction};
use athlesia_types::{Grid, Color, Action, PrimName, Params};
use athlesia_knowledge::KnowledgeBase;

fn grid_5x5_with_pixel(x: usize, y: usize, val: u8) -> Grid {
    let mut cells = vec![Color(0); 25];
    cells[y * 5 + x] = Color(val);
    Grid { width: 5, height: 5, cells }
}

fn grid_3x3_zeros() -> Grid {
    Grid { width: 3, height: 3, cells: vec![Color(0); 9] }
}

fn setup_wm() -> WorldModel {
    let initial = grid_5x5_with_pixel(0, 0, 1);
    let mut wm = WorldModel::new(initial.clone());
    wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);
    wm
}

#[test]
fn run_with_outcome_not_out_of_model() {
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let wm = WorldModel::new(grid_5x5_with_pixel(0, 0, 1)); // nincs hipotézis
    let prediction = wm.predict(&grid_5x5_with_pixel(0, 0, 1), &action);
    let observation = Observation { state: grid_5x5_with_pixel(0, 0, 1) };
    let mut kb = KnowledgeBase::new();

    let outcome = OpenWorldCycle::run_with_outcome(&wm, &action, &prediction, &observation, &mut kb);
    assert_eq!(outcome, OpenWorldOutcome::NotOutOfModel);
}

#[test]
fn run_with_outcome_abstain_when_low_confidence() {
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let prediction = Prediction { state: grid_3x3_zeros(), confidence: 0.5 };
    let observation = Observation { state: grid_5x5_with_pixel(0, 0, 1) };
    let mut kb = KnowledgeBase::new();

    // Itt a mismatch_score 1.0 lesz, mert dimenzióeltérés, tehát confidence >= 0.5 -> Verified.
    // Ez NEM jó Abstain tesztnek. Hogy Abstain legyen, olyan reziduális kell, ahol a mismatches kisebb.
    // Ehelyett a tesztet inkább úgy módosítjuk, hogy a `discover_candidate_concept` visszatérési
    // feltételét kikerüljük? Nem, a confidence 0.5 alatt kell lennie.
    // Készítünk egy olyan predikciót, amely ugyanolyan méretű és 1 pixel eltérés = 0.04 -> confidence 0.04 < 0.5.
    let pred_low = Prediction { state: grid_5x5_with_pixel(1, 0, 0), confidence: 0.5 };
    let obs_low = Observation { state: grid_5x5_with_pixel(0, 0, 1) };
    let outcome = OpenWorldCycle::run_with_outcome(&setup_wm(), &action, &pred_low, &obs_low, &mut kb);
    assert_eq!(outcome, OpenWorldOutcome::Abstain);
}

#[test]
fn run_with_outcome_verified_on_dimension_mismatch() {
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let prediction = Prediction { state: grid_3x3_zeros(), confidence: 0.5 };
    let observation = Observation { state: grid_5x5_with_pixel(0, 0, 1) };
    let mut kb = KnowledgeBase::new();

    let outcome = OpenWorldCycle::run_with_outcome(&setup_wm(), &action, &prediction, &observation, &mut kb);
    match outcome {
        OpenWorldOutcome::Verified(_) => {}
        other => panic!("Verified várt, de {:?} kaptunk", other),
    }
}
'''

write_file("crates/athlesia-core/tests/openworld_outcome_test.rs", test_code)
print("[2] openworld_outcome_test.rs létrehozva.")

# 3. Core tesztek futtatása
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

# 4. Teljes workspace teszt
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

# 5. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 11: add OpenWorldOutcome with explicit Abstain and Retrieved states"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
