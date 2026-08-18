#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Kernel lib.rs frissítése: importok bővítése
p = pathlib.Path("crates/athlesia-kernel/src/lib.rs")
s = p.read_text()

# Importok bővítése
if "use athlesia_world_model::{WorldModel, PredictionError};" in s:
    s = s.replace(
        "use athlesia_world_model::{WorldModel, PredictionError};",
        "use athlesia_world_model::{WorldModel, PredictionError, Observation};\nuse athlesia_core::openworld::{OpenWorldCycle, OpenWorldOutcome};",
        1,
    )
elif "use athlesia_world_model::WorldModel;" in s:
    s = s.replace(
        "use athlesia_world_model::WorldModel;",
        "use athlesia_world_model::{WorldModel, PredictionError, Observation};\nuse athlesia_core::openworld::{OpenWorldCycle, OpenWorldOutcome};",
        1,
    )
else:
    print("[ERROR] WorldModel import nem található.")
    sys.exit(1)

# 2. openworld_step metódus beszúrása az Agent impl végére
anchor = '''    /// Megerősített hipotézisek makrósítása.
    pub fn consolidate_learned_macros(&mut self) {
        for hyp in &self.wm.hypotheses {
            if hyp.status == athlesia_world_model::HypothesisStatus::Confirmed {
                self.kb.add_macro(
                    format!("learned_{}", self.kb.get_all_macros().len()),
                    hyp.program.clone(),
                );
                self.memory.long_term.add_program(hyp.program.clone());
            }
        }
    }
'''

new_method = anchor + '''

    /// Open-world megfigyelési lépés: predikció, kiértékelés és fogalomtanulás.
    ///
    /// A jelenlegi állapotból predikciót készít az adott akcióra, majd az
    /// `OpenWorldCycle::run_with_outcome` segítségével kiértékeli a
    /// megfigyelést, és ha OutOfModel, fogalomtanulást hajt végre.
    pub fn openworld_step(
        &mut self,
        action: &Action,
        observation: &Observation,
    ) -> OpenWorldOutcome {
        let current_state = self.wm.current_state.clone();
        let prediction = self.wm.predict(&current_state, action);
        OpenWorldCycle::run_with_outcome(
            &self.wm,
            action,
            &prediction,
            observation,
            &mut self.kb,
        )
    }
'''

if anchor not in s:
    print("[ERROR] consolidate_learned_macros blokk nem található.")
    sys.exit(1)

s = s.replace(anchor, new_method)
p.write_text(s)
print("[1] kernel lib.rs frissítve: openworld_step metódus hozzáadva.")

# 3. Új tesztfájl
test_code = r'''
use athlesia_kernel::Agent;
use athlesia_world_model::Observation;
use athlesia_types::{Grid, Color, Action, PrimName, Params};
use athlesia_core::openworld::{OpenWorldOutcome};

fn grid_5x5_with_pixel(x: usize, y: usize, val: u8) -> Grid {
    let mut cells = vec![Color(0); 25];
    cells[y * 5 + x] = Color(val);
    Grid { width: 5, height: 5, cells }
}

#[test]
fn agent_openworld_step_creates_verified_concept_on_out_of_model() {
    let initial = grid_5x5_with_pixel(0, 0, 1);
    let mut agent = Agent::new(initial.clone());

    // Irreleváns hipotézis, hogy a Translate akcióra OutOfModel legyen.
    agent.wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);

    let action = Action {
        prim: PrimName::Translate,
        params: Params::Translate(0, 1),
    };
    // Szándékosan eltérő dimenziójú megfigyelés -> magas mismatch_score -> Verified
    let observation = Observation {
        state: Grid::new(3, 3),
    };

    let outcome = agent.openworld_step(&action, &observation);

    match outcome {
        OpenWorldOutcome::Verified(_) | OpenWorldOutcome::Retrieved(_) => {}
        other => panic!("Várt Verified/Retrieved, de {:?} kaptunk", other),
    }

    assert_eq!(
        agent.kb.get_verified_concepts().len(),
        1,
        "Egy igazolt fogalomnak kell keletkeznie"
    );
}
'''

write_file("crates/athlesia-kernel/tests/openworld_agent_test.rs", test_code)
print("[2] openworld_agent_test.rs létrehozva.")

# 4. Kernel tesztek futtatása
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
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 14: Agent.openworld_step integrates WorldModel prediction with OpenWorldCycle"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
