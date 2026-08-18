#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. WorldModel bővítése recent_errors mezővel és metódussal
p = pathlib.Path("crates/athlesia-world-model/src/lib.rs")
s = p.read_text()

# Hozzáadjuk a mezőt a struct-hoz
struct_marker = "pub struct WorldModel {\n    pub current_state: State,\n    pub hypotheses: Vec<TransitionHypothesis>,\n    pub tick: u64,\n}"
new_struct = "pub struct WorldModel {\n    pub current_state: State,\n    pub hypotheses: Vec<TransitionHypothesis>,\n    pub tick: u64,\n    pub recent_errors: Vec<PredictionError>,\n}"
if struct_marker not in s:
    print("[ERROR] WorldModel struct nem található a várt formában.")
    sys.exit(1)
s = s.replace(struct_marker, new_struct)

# new() inicializálás kiegészítése
new_method_old = "WorldModel {\n            current_state: initial_grid,\n            hypotheses: Vec::new(),\n            tick: 0,\n        }"
new_method_new = "WorldModel {\n            current_state: initial_grid,\n            hypotheses: Vec::new(),\n            tick: 0,\n            recent_errors: Vec::new(),\n        }"
if new_method_old not in s:
    print("[ERROR] WorldModel::new inicializálás nem található.")
    sys.exit(1)
s = s.replace(new_method_old, new_method_new)

# record_prediction_error metódus hozzáadása a WorldModel impl blokk végére
# A impl blokk a learn_from_error után ér véget, beszúrjuk elé
method_insert = '''
    /// A predikciós hiba tárolása későbbi absztrakcióhoz.
    pub fn record_prediction_error(&mut self, error: PredictionError) {
        self.recent_errors.push(error);
    }
'''
# A learn_from_error függvény végéhez illesztjük, a záró kapcsos zárójel elé
# Egyszerűen a "    pub fn learn_from_error" előtti részhez? Inkább a függvény után.
# Keresünk egy egyedi mintát: "    pub fn learn_from_error" és utána a záró
old_learn_end = '''    pub fn learn_from_error(&mut self, hypothesis_id: u64, _error: &PredictionError) {
        if let Some(hyp) = self.hypotheses.iter_mut().find(|h| h.id == hypothesis_id) {
            hyp.evidence_against += 1;
            hyp.status = HypothesisStatus::Falsified;
        }
    }
'''
new_learn_end = '''    pub fn learn_from_error(&mut self, hypothesis_id: u64, _error: &PredictionError) {
        if let Some(hyp) = self.hypotheses.iter_mut().find(|h| h.id == hypothesis_id) {
            hyp.evidence_against += 1;
            hyp.status = HypothesisStatus::Falsified;
        }
    }

    /// A predikciós hiba tárolása későbbi absztrakcióhoz.
    pub fn record_prediction_error(&mut self, error: PredictionError) {
        self.recent_errors.push(error);
    }
'''
if old_learn_end not in s:
    print("[ERROR] learn_from_error blokk nem található.")
    sys.exit(1)
s = s.replace(old_learn_end, new_learn_end)

write_file(p, s)
print("[1] WorldModel frissítve: recent_errors mező és record_prediction_error metódus.")

# 2. kernel lib.rs módosítása a predikciós hiba rögzítéséhez
p = pathlib.Path("crates/athlesia-kernel/src/lib.rs")
s = p.read_text()

# Import hozzáadása
if "use athlesia_world_model::PredictionError;" not in s:
    s = s.replace("use athlesia_world_model::WorldModel;", "use athlesia_world_model::{WorldModel, PredictionError};")

# A training loop-ban a verifikáció utáni rész módosítása
old_verify_block = '''        // Verifikáció
        let verifier = Verifier::new();
        if verifier.verify(&program, &vec![(input_grid.clone(), output_grid.clone())]) == VerificationResult::Accept {
            let id = agent.core.known_programs.len() as u64;
            agent.core.known_programs.push(program.clone());
            agent.core.meta.record_success_in_context(
                athlesia_features::extract_features(&input_grid),
                id,
            );
            agent.memory.append_episode(input_grid.clone(), output_grid.clone(), program);
        }
'''
new_verify_block = '''        // Verifikáció
        let verifier = Verifier::new();
        if verifier.verify(&program, &vec![(input_grid.clone(), output_grid.clone())]) == VerificationResult::Accept {
            let id = agent.core.known_programs.len() as u64;
            agent.core.known_programs.push(program.clone());
            agent.core.meta.record_success_in_context(
                athlesia_features::extract_features(&input_grid),
                id,
            );
            agent.memory.append_episode(input_grid.clone(), output_grid.clone(), program);
        } else {
            // Predikciós hiba rögzítése
            let mut budget = Budget { max_steps: 1000, max_depth: 100 };
            let actual_output = athlesia_executor::run_program(&program, &input_grid, &mut budget)
                .unwrap_or_else(|_| input_grid.clone());
            let report = verifier.report(&program, &input_grid, &output_grid);
            let error = PredictionError {
                expected: output_grid.clone(),
                observed: actual_output,
                summary: report.failure_signature.summary.clone(),
                feature_mismatch: report.failure_signature.pixel_mismatch,
            };
            if let Some(hyp) = agent.wm.hypotheses.iter().find(|h| h.program == program) {
                let id = hyp.id;
                agent.wm.learn_from_error(id, &error);
            }
            agent.wm.record_prediction_error(error);
        }
'''
if old_verify_block not in s:
    print("[ERROR] A verifikációs blokk nem található a várt formában.")
    sys.exit(1)
s = s.replace(old_verify_block, new_verify_block)

write_file(p, s)
print("[2] kernel lib.rs frissítve: predikciós hibák rögzítése.")

# 3. Új teszt a world-model számára
test_code = r'''
use athlesia_world_model::{WorldModel, PredictionError};
use athlesia_types::{Grid, Color};

fn grid_5x5_filled(value: u8) -> Grid {
    Grid {
        width: 5,
        height: 5,
        cells: vec![Color(value); 25],
    }
}

#[test]
fn record_prediction_error_stores_error() {
    let mut wm = WorldModel::new(grid_5x5_filled(0));
    let error = PredictionError {
        expected: grid_5x5_filled(1),
        observed: grid_5x5_filled(2),
        summary: "test mismatch".to_string(),
        feature_mismatch: 25,
    };
    wm.record_prediction_error(error);
    assert_eq!(wm.recent_errors.len(), 1);
    assert_eq!(wm.recent_errors[0].summary, "test mismatch");
}
'''
write_file("crates/athlesia-world-model/tests/prediction_error_test.rs", test_code)
print("[3] prediction_error_test.rs létrehozva.")

# 4. Tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-world-model"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] World-model tesztek nem mentek át.")
    sys.exit(1)

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

print("\n[SUCCESS] Phase 4 tesztek zöldek.")

# 5. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 4: record structured prediction errors in world model and kernel"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
