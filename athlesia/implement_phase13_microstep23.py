#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# Új tesztfájl a valódi interaktív transzferhez
test_code = r'''
use athlesia_core::openworld::{OpenWorldCycle, OpenWorldOutcome};
use athlesia_world_model::{WorldModel, Observation};
use athlesia_knowledge::KnowledgeBase;
use athlesia_metalearner::MetaLearner;
use athlesia_interactive::{Environment, ProbeAction};
use athlesia_planner::ExperimentRequest;
use athlesia_types::{Grid, Action, PrimName, Params};

fn create_world_model_with_reflect_only() -> WorldModel {
    let initial = Grid::new(5, 5);
    let mut wm = WorldModel::new(initial);
    wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);
    wm
}

#[test]
fn openworld_interactive_transfer_between_episodes() {
    // --- 1. epizód: felfedezés valós környezetben ---
    let mut env1 = Environment::new(ProbeAction::C);
    let wm1 = create_world_model_with_reflect_only();
    let mut kb = KnowledgeBase::new();
    let mut meta = MetaLearner::new();

    let request = ExperimentRequest {
        action: Action { prim: PrimName::Translate, params: Params::Translate(0, 1) },
        target_hypothesis: "object_position_change(A,B)".to_string(),
        expected_observation: "object_position_change(A,B)".to_string(),
    };

    let outcome1 = OpenWorldCycle::run_experiment_cycle(
        &wm1,
        &mut kb,
        &mut meta,
        request.clone(),
        |_| {
            let observed_grid = env1.step(&ProbeAction::C);
            Observation { state: observed_grid }
        },
    );

    match outcome1 {
        OpenWorldOutcome::Verified(_) => {}
        other => panic!("Első epizódban Verified várt, de {:?} kaptunk", other),
    }
    assert_eq!(kb.get_verified_concepts().len(), 1);

    // --- 2. epizód: ugyanaz a fogalom, új környezet ---
    let mut env2 = Environment::new(ProbeAction::C);
    let wm2 = create_world_model_with_reflect_only();

    let outcome2 = OpenWorldCycle::run_experiment_cycle(
        &wm2,
        &mut kb,
        &mut meta,
        request,
        |_| {
            let observed_grid = env2.step(&ProbeAction::C);
            Observation { state: observed_grid }
        },
    );

    match outcome2 {
        OpenWorldOutcome::Retrieved(_) => {}
        other => panic!("Második epizódban Retrieved várt, de {:?} kaptunk", other),
    }

    // A fogalom nem duplikálódik.
    assert_eq!(
        kb.get_verified_concepts().len(),
        1,
        "A második epizódnak a meglévő fogalmat kell visszaadnia, nem újat létrehozni"
    );
}
'''

write_file("crates/athlesia-core/tests/openworld_interactive_transfer_test.rs", test_code)
print("[1] openworld_interactive_transfer_test.rs létrehozva.")

# Core tesztek futtatása
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

# Teljes workspace teszt
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

# Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 23: demonstrate open-world interactive transfer between episodes"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
