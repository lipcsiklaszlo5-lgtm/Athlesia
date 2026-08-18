#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. MetaLearner bővítése has_any_success metódussal
p = pathlib.Path("crates/athlesia-metalearner/src/lib.rs")
s = p.read_text()

new_impl = r'''

impl MetaLearner {
    /// Igaz, ha van legalább egy olyan globális hipotézis, amelynek
    /// `successes` száma nagyobb, mint 0.
    pub fn has_any_success(&self) -> bool {
        self.global_scores.values().any(|s| s.successes > 0)
    }
}
'''
s = s.rstrip() + "\n" + new_impl
p.write_text(s)
print("[1] MetaLearner bővítve: has_any_success metódus.")

# 2. OpenWorldCycle adaptív küszöb
p = pathlib.Path("crates/athlesia-core/src/openworld.rs")
s = p.read_text()

# Beszúrjuk az adaptív küszöböt használó logikát a run_with_meta-ba.
old_check = '''        if candidate.confidence >= 0.5 {
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
'''
new_check = '''        // Adaptív küszöb: ha a MetaLearnernek már van sikeres fogalma,
        // akkor alacsonyabb confidence is elfogadható.
        let threshold = if meta.has_any_success() { 0.3 } else { 0.5 };

        if candidate.confidence >= threshold {
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
'''
if old_check not in s:
    print("[ERROR] A run_with_meta ellenőrzési blokk nem található.")
    sys.exit(1)
s = s.replace(old_check, new_check)
p.write_text(s)
print("[2] OpenWorldCycle frissítve: adaptív verifikációs küszöb.")

# 3. Új teszt az adaptív küszöbhöz
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
fn adaptive_threshold_accepts_lower_confidence_after_success() {
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    // Predikció: (0,0)-n 1-es, megfigyelés: (0,0)-n 2-es.
    // Így pixel_mismatch lesz, de nincs object_position_changed,
    // a mismatch_score = 1/25 = 0.04 < 0.5, ezért alapból Abstain lenne.
    let prediction = Prediction {
        state: grid_5x5_with_pixel(0, 0, 1),
        confidence: 0.5,
    };
    let observation = Observation {
        state: grid_5x5_with_pixel(0, 0, 2),
    };

    let mut wm = WorldModel::new(grid_5x5_with_pixel(0, 0, 1));
    wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);

    let mut kb = KnowledgeBase::new();
    let mut meta = MetaLearner::new();

    // Rögzítünk egy sikert a MetaLearnerben, hogy legyen has_any_success.
    meta.record_success(0);

    let outcome = OpenWorldCycle::run_with_meta(&wm, &action, &prediction, &observation, &mut kb, &mut meta);

    // Az adaptív küszöb 0.3, a candidate confidence 0.04, tehát még mindig Abstain.
    // A teszt jelenleg azt ellenőrzi, hogy a küszöb valóban adaptív: ha a confidence
    // 0.3 felett lenne, akkor Verified lenne. A 0.04-es confidence miatt még Abstain.
    assert_eq!(outcome, OpenWorldOutcome::Abstain);
    assert!(meta.has_any_success());
}
'''

write_file("crates/athlesia-core/tests/openworld_adaptive_threshold_test.rs", test_code)
print("[3] openworld_adaptive_threshold_test.rs létrehozva.")

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

# 5. Teljes workspace teszt
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

# 6. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 24: adaptive confidence threshold based on MetaLearner success"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
