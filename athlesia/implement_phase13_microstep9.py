#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. openworld.rs cseréje az új recall logikával
p = pathlib.Path("crates/athlesia-core/src/openworld.rs")
s = p.read_text()

start = s.find("impl OpenWorldCycle")
if start == -1:
    print("[ERROR] impl OpenWorldCycle nem található.")
    sys.exit(1)

new_impl = r'''impl OpenWorldCycle {
    /// Lefuttatja a Phase 13 alapciklust.
    ///
    /// 1. Kiértékeli a predikciót (tudásállapot + reziduális).
    /// 2. Ha OutOfModel, a reziduálisból candidate conceptet generál.
    /// 3. Ha a candidate relation_pattern már létezik a KB-ben, visszaadja azt.
    /// 4. Különben, ha a candidate confidence elég magas, verifikálja és beilleszti a KB-be.
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

        // Transfer: ha már van ilyen kapcsolati mintánk, használjuk azt.
        if let Some(existing) = kb
            .get_verified_concepts()
            .iter()
            .find(|c| c.relation_pattern == candidate.sketch.relation_pattern)
        {
            return Some(existing.clone());
        }

        // Egyszerű verifikáció: ha a candidate confidence elér egy küszöböt,
        // igazolt fogalomként tároljuk.
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

s = s[:start] + new_impl
p.write_text(s)
print("[1] openworld.rs frissítve: recall/transfer logika hozzáadva.")

# 2. Új tesztfájl
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
fn openworld_cycle_recalls_existing_concept_on_same_pattern() {
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let prediction = Prediction {
        state: grid_3x3_zeros(),
        confidence: 0.5,
    };
    let observation = Observation {
        state: grid_5x5_with_pixel(0, 0, 1),
    };

    let mut kb = KnowledgeBase::new();

    // Első alkalom: létrehoz egy igazolt fogalmat.
    let first = OpenWorldCycle::run(&setup_wm(), &action, &prediction, &observation, &mut kb);
    assert!(first.is_some());
    let first_count = kb.get_verified_concepts().len();
    assert_eq!(first_count, 1);

    // Második, új példány azonos reziduális mintával:
    // ugyanazt a fogalmat kell visszakapnia, nem szabad új elemet hozzáadnia.
    let second = OpenWorldCycle::run(&setup_wm(), &action, &prediction, &observation, &mut kb);
    assert!(second.is_some());
    assert_eq!(kb.get_verified_concepts().len(), first_count);
    assert_eq!(
        second.as_ref().unwrap().relation_pattern,
        first.as_ref().unwrap().relation_pattern
    );
}
'''
write_file("crates/athlesia-core/tests/openworld_transfer_test.rs", test_code)
print("[2] openworld_transfer_test.rs létrehozva.")

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
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 9: recall existing verified concept by relation_pattern"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
