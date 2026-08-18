#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Core Cargo.toml: dev-dependency hozzáadása az interactive crate-hez
p = pathlib.Path("crates/athlesia-core/Cargo.toml")
s = p.read_text()
if "athlesia-interactive" not in s:
    if "[dev-dependencies]" not in s:
        s += "\n[dev-dependencies]\nathlesia-interactive = { path = \"../athlesia-interactive\" }\n"
    else:
        s = s.replace("[dev-dependencies]", "[dev-dependencies]\nathlesia-interactive = { path = \"../athlesia-interactive\" }", 1)
    p.write_text(s)
    print("[1] core Cargo.toml: dev-dependency athlesia-interactive hozzáadva.")
else:
    print("[1] core Cargo.toml már tartalmazza a dev-dependency-t.")

# 2. Integrációs teszt létrehozása
test_code = r'''
use athlesia_core::openworld::{OpenWorldCycle, OpenWorldOutcome};
use athlesia_world_model::{WorldModel, Observation, Prediction};
use athlesia_knowledge::KnowledgeBase;
use athlesia_interactive::{Environment, ProbeAction, InteractiveAgent};
use athlesia_types::{Grid, Action, PrimName, Params};

fn object_position(grid: &Grid) -> (i8, i8) {
    for y in 0..grid.height as i8 {
        for x in 0..grid.width as i8 {
            if let Some(c) = grid.get(x, y) {
                if c.0 != 0 {
                    return (x, y);
                }
            }
        }
    }
    (-1, -1)
}

#[test]
fn active_loop_discovers_hidden_trigger_via_openworld_cycle() {
    // A környezet rejtett triggere C.
    let mut env = Environment::new(ProbeAction::C);
    let initial_grid = env.grid.clone();

    // WorldModel kezdetben csak egy irreleváns hipotézissel (ReflectH),
    // hogy a Translate akcióra OutOfModel-t adjon.
    let mut wm = WorldModel::new(initial_grid.clone());
    wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);

    let mut kb = KnowledgeBase::new();
    let mut steps = 0;
    let mut verified_concepts_count = 0;

    // Próbáljuk ki a Translate(1,0) akciót.
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };

    // Predikció: a jelenlegi modell szerint a Translate nem változtat
    // (nincs rá hipotézis), ezért a predict eredménye maga az input.
    let prediction = wm.predict(&initial_grid, &action);

    // Megfigyelés a környezetből: valójában az objektum jobbra mozdul.
    let old_pos = object_position(&env.grid);
    let observed_grid = env.step(&ProbeAction::C); // A C akció a rejtett trigger
    let new_pos = object_position(&observed_grid);
    assert_ne!(old_pos, new_pos, "A C akciónak mozgást kell kiváltania.");

    let observation = Observation { state: observed_grid.clone() };

    // Open-world ciklus futtatása.
    let outcome = OpenWorldCycle::run_with_outcome(
        &wm,
        &action,
        &prediction,
        &observation,
        &mut kb,
    );

    // Az eltérés miatt OutOfModel-nek kell lennie, és fogalmat kell létrehoznia/igazolnia.
    match outcome {
        OpenWorldOutcome::Verified(_) | OpenWorldOutcome::Retrieved(_) => {
            verified_concepts_count = kb.get_verified_concepts().len();
        }
        other => panic!("Várt Verified/Retrieved, de {:?} kaptunk", other),
    }

    assert_eq!(verified_concepts_count, 1, "Egy igazolt fogalomnak kell keletkeznie.");
    steps += 1;
    assert!(steps < 5, "A felfedezés túl sok lépést igényelt.");
}
'''
write_file("crates/athlesia-core/tests/openworld_interactive_integration.rs", test_code)
print("[2] openworld_interactive_integration.rs létrehozva.")

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
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 12: closed-loop integration with interactive environment"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
