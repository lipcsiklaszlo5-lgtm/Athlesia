#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Core openworld.rs frissítése: run_with_meta hozzáadása, régi run_with_outcome megtartása
p = pathlib.Path("crates/athlesia-core/src/openworld.rs")
s = p.read_text()

# import MetaLearner
if "use athlesia_metalearner::MetaLearner;" not in s:
    s = s.replace(
        "use athlesia_knowledge::KnowledgeBase;",
        "use athlesia_knowledge::KnowledgeBase;\nuse athlesia_metalearner::MetaLearner;",
        1,
    )

# A run_with_outcome metódus elé beszúrjuk a run_with_meta-t
old_run = '''    /// Az open-world ciklus kimenettel együtt.
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
    ) -> OpenWorldOutcome {'''

new_run = '''    /// Open-world ciklus MetaLearner integrációval.
    ///
    /// Ha a candidate relation_pattern szerepel a MetaLearner
    /// `failed_concepts` archívumában, azonnal Abstain.
    /// Ha a candidate confidence < 0.5, rögzíti a kudarcot.
    pub fn run_with_meta(
        wm: &WorldModel,
        action: &Action,
        prediction: &Prediction,
        observation: &Observation,
        kb: &mut KnowledgeBase,
        meta: &mut MetaLearner,
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

        // Ismert kudarc ellenőrzése a MetaLearner archívumban.
        if meta.is_known_failed_concept(&candidate.sketch.relation_pattern) {
            return OpenWorldOutcome::Abstain;
        }

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
            // Kudarc rögzítése a MetaLearner archívumban.
            meta.record_failed_concept(candidate.sketch.relation_pattern.clone());
            OpenWorldOutcome::Abstain
        }
    }

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
        // A régi metódus meghívja az újat egy friss MetaLearnerrel,
        // hogy a korábbi tesztek változatlanok maradjanak.
        let mut meta = MetaLearner::new();
        Self::run_with_meta(wm, action, prediction, observation, kb, &mut meta)
    }'''

if old_run not in s:
    print("[ERROR] A run_with_outcome blokk nem található.")
    sys.exit(1)

s = s.replace(old_run, new_run)
p.write_text(s)
print("[1] openworld.rs frissítve: run_with_meta és MetaLearner integráció.")

# 2. Kernel openworld_step frissítése, hogy a run_with_meta-t hívja
p = pathlib.Path("crates/athlesia-kernel/src/lib.rs")
s = p.read_text()

old_step = '''        OpenWorldCycle::run_with_outcome(
            &self.wm,
            action,
            &prediction,
            observation,
            &mut self.kb,
        )'''
new_step = '''        OpenWorldCycle::run_with_meta(
            &self.wm,
            action,
            &prediction,
            observation,
            &mut self.kb,
            &mut self.core.meta,
        )'''

if old_step not in s:
    print("[ERROR] openworld_step hívása nem található.")
    sys.exit(1)

s = s.replace(old_step, new_step)
p.write_text(s)
print("[2] kernel lib.rs frissítve: openworld_step a run_with_meta-t használja.")

# 3. Új teszt a MetaLearner archívum használatára
test_code = r'''
use athlesia_core::openworld::{OpenWorldCycle, OpenWorldOutcome};
use athlesia_world_model::{WorldModel, Observation, Prediction};
use athlesia_knowledge::KnowledgeBase;
use athlesia_metalearner::MetaLearner;
use athlesia_types::{Grid, Color, Action, PrimName, Params};

fn grid_5x5_with_pixel(x: usize, y: usize, val: u8) -> Grid {
    let mut cells = vec![Color(0); 25];
    cells[y * 5 + x] = Color(val);
    Grid { width: 5, height: 5, cells }
}

#[test]
fn run_with_meta_records_failed_concept_and_abstains_on_second_try() {
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let prediction_low = Prediction {
        state: grid_5x5_with_pixel(1, 0, 0),
        confidence: 0.5,
    };
    let observation = Observation {
        state: grid_5x5_with_pixel(0, 0, 1),
    };
    let mut wm = WorldModel::new(grid_5x5_with_pixel(0, 0, 1));
    wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);

    let mut kb = KnowledgeBase::new();
    let mut meta = MetaLearner::new();

    // Első próba: alacsony confidence -> Abstain, kudarc rögzítése.
    let outcome1 = OpenWorldCycle::run_with_meta(
        &wm, &action, &prediction_low, &observation, &mut kb, &mut meta,
    );
    assert_eq!(outcome1, OpenWorldOutcome::Abstain);
    assert!(meta.is_known_failed_concept("pixel_mismatch"));

    // Második próba ugyanazzal a mintával: az archívum miatt Abstain.
    let outcome2 = OpenWorldCycle::run_with_meta(
        &wm, &action, &prediction_low, &observation, &mut kb, &mut meta,
    );
    assert_eq!(outcome2, OpenWorldOutcome::Abstain);
    assert_eq!(kb.get_verified_concepts().len(), 0);
}
'''
write_file("crates/athlesia-core/tests/openworld_meta_archive_test.rs", test_code)
print("[3] openworld_meta_archive_test.rs létrehozva.")

# 4. Core tesztek futtatása
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

# 5. Kernel tesztek futtatása (a openworld_step változása miatt)
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-kernel"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Kernel tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Kernel tesztek zöldek.")

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
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 18: integrate MetaLearner failed-concept archive into OpenWorldCycle"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
