#!/usr/bin/env python3
import os, subprocess, sys, pathlib

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. WorldModel lib.rs kiegészítése Belief és PredictionError típusokkal
p = pathlib.Path("crates/athlesia-world-model/src/lib.rs")
s = p.read_text()

# Belief struktúra hozzáadása
belief_code = '''
/// Explicit prior/belief egy fogalom vagy szabály megbízhatóságáról.
#[derive(Debug, Clone)]
pub struct Belief {
    pub concept_id: u64,
    pub confidence: f32,
    pub evidence_for: usize,
    pub evidence_against: usize,
}

/// Predikciós hiba: miért nem egyezett a predikció a megfigyeléssel.
#[derive(Debug, Clone)]
pub struct PredictionError {
    pub expected: Grid,
    pub observed: Grid,
    pub summary: String,
    pub feature_mismatch: usize,
}

impl WorldModel {
    /// A modell frissítése predikciós hiba alapján.
    /// A hibát egy hipotézishez kötjük, és csökkentjük annak konfidenciáját.
    pub fn learn_from_error(&mut self, hypothesis_id: u64, error: &PredictionError) {
        if let Some(hyp) = self.hypotheses.iter_mut().find(|h| h.id == hypothesis_id) {
            hyp.evidence_against += 1;
            hyp.status = HypothesisStatus::Falsified;
        }
    }
}
'''
# Beszúrás a WorldModel struct után
anchor = "pub struct WorldModel {"
idx = s.find(anchor)
if idx == -1:
    print("[ERROR] WorldModel struct nem található")
    sys.exit(1)

# Belief és PredictionError a struct előtt, learn_from_error az impl blokk után
s = s[:idx] + belief_code + s[idx:]

write_file(p, s)
print("[1] WorldModel Belief, PredictionError és learn_from_error hozzáadva.")

# 2. Teszt hozzáadása
write_file("crates/athlesia-world-model/tests/belief_test.rs", r'''
use athlesia_world_model::{WorldModel, PredictionError};
use athlesia_types::{Grid, PrimName, Params};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn learn_from_error_falsifies_hypothesis() {
    let mut wm = WorldModel::new(build_grid([[0; 5]; 5]));
    let program = vec![(PrimName::Translate, Params::Translate(1, 0))];
    let id = wm.add_hypothesis(program);

    let error = PredictionError {
        expected: build_grid([[0,1,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]),
        observed: build_grid([[1,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]),
        summary: "direction mismatch".to_string(),
        feature_mismatch: 2,
    };

    wm.learn_from_error(id, &error);
    assert_eq!(wm.hypotheses[0].evidence_against, 1);
    assert_eq!(wm.hypotheses[0].status, athlesia_world_model::HypothesisStatus::Falsified);
}
''')

print("[2] belief_test.rs létrehozva.")

# 3. Tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-world-model", "--test", "belief_test"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Belief tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Belief tesztek zöldek.")

# 4. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Add belief and prediction error learning to WorldModel"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
