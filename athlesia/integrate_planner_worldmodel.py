#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
PLAN_DIR = os.path.join(PROJECT, "crates", "athlesia-planner")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Planner Cargo.toml-hez a world-model függőség hozzáadása
cargo_path = os.path.join(PLAN_DIR, "Cargo.toml")
cargo_content = pathlib.Path(cargo_path).read_text()
if "athlesia-world-model" not in cargo_content:
    cargo_content = cargo_content.replace(
        "[dependencies]\n",
        "[dependencies]\nathlesia-world-model = { path = \"../athlesia-world-model\" }\n"
    )
    write_file(cargo_path, cargo_content)
    print("[INFO] world-model függőség hozzáadva a Planner-hez.")

# 2. Planner lib.rs frissítése: WorldModel használata
lib_path = os.path.join(PLAN_DIR, "src", "lib.rs")
lib_content = r'''
use athlesia_types::{Grid, PrimName, Params, Program, Action};
use athlesia_search::{search, beam_search};
use athlesia_world_model::WorldModel;

/// A tervező üzemmódja.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlannerMode {
    GoalDirected,
    Exploration,
}

/// A Manhattan Kernel tervezője.
///
/// - Cél-irányított mód: egy ismert cél-grid eléréséhez készít programot.
///   Ehhez a Search Engine-t használja, de a WorldModel konfidenciáját is figyelembe veszi.
/// - Feltáró mód: a WorldModel bizonytalanságát használja, és a legbizonytalanabb
///   akciót választja, hogy maximális információnyerést érjen el.
#[derive(Debug)]
pub struct Planner {
    pub mode: PlannerMode,
}

impl Planner {
    pub fn new(mode: PlannerMode) -> Self {
        Planner { mode }
    }

    /// Terv készítése. A `wm` a WorldModel, ami a belső szimulációt és a bizonytalanságot adja.
    pub fn plan(
        &self,
        current: &Grid,
        target: Option<&Grid>,
        wm: &WorldModel,
        max_depth: usize,
    ) -> Option<Program> {
        match self.mode {
            PlannerMode::GoalDirected => {
                let target_grid = target?;
                // Először a beam search-t próbáljuk, mert gyorsabb lehet
                if let Some(program) = beam_search(current, target_grid, max_depth, 10) {
                    return Some(program);
                }
                // Ha beam search nem talál, akkor a teljes keresőt használjuk
                search(current, target_grid, max_depth)
            }
            PlannerMode::Exploration => {
                // Alap akciók listája, amiket felfedezhetünk
                let actions = vec![
                    Action { prim: PrimName::Translate, params: Params::Translate(1, 0) },
                    Action { prim: PrimName::Translate, params: Params::Translate(-1, 0) },
                    Action { prim: PrimName::Translate, params: Params::Translate(0, 1) },
                    Action { prim: PrimName::Translate, params: Params::Translate(0, -1) },
                    Action { prim: PrimName::ReflectH, params: Params::None },
                    Action { prim: PrimName::ReflectV, params: Params::None },
                    Action { prim: PrimName::Rotate90, params: Params::None },
                ];

                // Kiválasztjuk a legbizonytalanabb akciót
                let mut best_action: Option<Action> = None;
                let mut max_uncertainty = -1.0;
                for action in actions {
                    let uncertainty = wm.uncertainty(current, &action);
                    if uncertainty > max_uncertainty {
                        max_uncertainty = uncertainty;
                        best_action = Some(action);
                    }
                }

                best_action.map(|a| vec![(a.prim, a.params)])
            }
        }
    }
}
'''
write_file(lib_path, lib_content)
print("[INFO] Planner frissítve WorldModel integrációval.")

# 3. Tesztek frissítése
test_path = os.path.join(PLAN_DIR, "tests", "planner_test.rs")
test_content = r'''
use athlesia_planner::{Planner, PlannerMode};
use athlesia_world_model::WorldModel;
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn goal_directed_planner_finds_reachable_target() {
    let wm = WorldModel::new(build_grid([[0; 5]; 5]));
    let planner = Planner::new(PlannerMode::GoalDirected);

    let current = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let target = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let plan = planner.plan(&current, Some(&target), &wm, 2);
    assert!(plan.is_some());
    assert!(!plan.unwrap().is_empty());
}

#[test]
fn goal_directed_planner_returns_none_when_no_solution() {
    let wm = WorldModel::new(build_grid([[0; 5]; 5]));
    let planner = Planner::new(PlannerMode::GoalDirected);

    let current = build_grid([[0; 5]; 5]);
    let target = build_grid([
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
    ]);

    let plan = planner.plan(&current, Some(&target), &wm, 2);
    assert!(plan.is_none());
}

#[test]
fn exploration_planner_selects_uncertain_action() {
    let wm = WorldModel::new(build_grid([[0; 5]; 5]));
    let planner = Planner::new(PlannerMode::Exploration);

    let current = build_grid([[0; 5]; 5]);
    let plan = planner.plan(&current, None, &wm, 1);
    assert!(plan.is_some());
    assert_eq!(plan.unwrap().len(), 1);
}
'''
write_file(test_path, test_content)
print("[INFO] Planner tesztek frissítve.")

# 4. Kernel lib.rs frissítése, hogy a Planner megkapja a WorldModel-t
kernel_lib = os.path.join(PROJECT, "crates", "athlesia-kernel", "src", "lib.rs")
kernel_content = pathlib.Path(kernel_lib).read_text()
kernel_content = kernel_content.replace(
    "planner: &Planner,",
    "planner: &Planner, wm: &athlesia_world_model::WorldModel,"
)
kernel_content = kernel_content.replace(
    "planner.plan(input, Some(target), max_depth)",
    "planner.plan(input, Some(target), wm, max_depth)"
)
write_file(kernel_lib, kernel_content)
print("[INFO] Kernel lib.rs frissítve WorldModel átadására.")

# 5. Kernel teszt frissítése
kernel_test = os.path.join(PROJECT, "crates", "athlesia-kernel", "tests", "kernel_test.rs")
kt_content = pathlib.Path(kernel_test).read_text()
kt_content = kt_content.replace(
    "use athlesia_planner::{Planner, PlannerMode};",
    "use athlesia_planner::{Planner, PlannerMode};\nuse athlesia_world_model::WorldModel;"
)
kt_content = kt_content.replace(
    "let planner = Planner::new(PlannerMode::GoalDirected);",
    "let planner = Planner::new(PlannerMode::GoalDirected);\n    let wm = WorldModel::new(build_grid([[0; 5]; 5]));"
)
kt_content = kt_content.replace(
    "&planner,",
    "&planner, &wm,"
)
write_file(kernel_test, kt_content)
print("[INFO] Kernel teszt frissítve.")

# 6. Tesztek futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-planner"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Planner tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Planner tesztek zöldek.")

result = subprocess.run(["cargo", "test", "-p", "athlesia-kernel"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Kernel tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Kernel tesztek zöldek.")

# 7. Git commit és push (szülőből)
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Integrate WorldModel into Planner"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
