#!/usr/bin/env python3
import os, subprocess, sys, pathlib

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Planner lib.rs teljes újraírása a dokumentum szerint
write_file("crates/athlesia-planner/src/lib.rs", r'''
use athlesia_types::{Grid, PrimName, Params, Program, Action};
use athlesia_search::{SearchEngine, DefaultSearchEngine, SearchStrategy};
use athlesia_world_model::{WorldModel, Query};

/// A tervező üzemmódja.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlannerMode {
    GoalDirected,
    Exploration,
}

/// A Manhattan Kernel tervezője.
///
/// Cél-irányított mód: a Search Engine-t használja a cél eléréséhez.
/// Feltáró mód: a WorldModel bizonytalanságát használva a legbizonytalanabb
/// akciót választja, hogy maximális információnyerést érjen el.
#[derive(Debug)]
pub struct Planner {
    pub mode: PlannerMode,
}

impl Planner {
    pub fn new(mode: PlannerMode) -> Self {
        Planner { mode }
    }

    /// Terv készítése az aktuális állapotból a cél eléréséhez.
    ///
    /// - Ha `target` megadott és a mód `GoalDirected`, a Search Engine-t hívja.
    /// - Ha `target` nincs, vagy a mód `Exploration`, a legbizonytalanabb akciót adja.
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
                let engine = DefaultSearchEngine;
                // A dokumentum szerint a célfüggvény a cél elérése,
                // most a Search Engine általános keresését használjuk.
                engine.search(current, target_grid, max_depth, SearchStrategy::AStar)
            }
            PlannerMode::Exploration => {
                // Alap akciók listája, amiket felfedezhetünk.
                let actions = vec![
                    Action { prim: PrimName::Translate, params: Params::Translate(1, 0) },
                    Action { prim: PrimName::Translate, params: Params::Translate(-1, 0) },
                    Action { prim: PrimName::Translate, params: Params::Translate(0, 1) },
                    Action { prim: PrimName::Translate, params: Params::Translate(0, -1) },
                    Action { prim: PrimName::ReflectH, params: Params::None },
                    Action { prim: PrimName::ReflectV, params: Params::None },
                    Action { prim: PrimName::Rotate90, params: Params::None },
                    Action { prim: PrimName::Rotate180, params: Params::None },
                    Action { prim: PrimName::Rotate270, params: Params::None },
                    Action { prim: PrimName::SwapColors, params: Params::SwapColors(1, 2) },
                ];

                // Kiválasztjuk a legbizonytalanabb akciót.
                let mut best_action: Option<Action> = None;
                let mut max_uncertainty = -1.0;
                for action in actions {
                    let query = Query {
                        state: current.clone(),
                        action,
                    };
                    let uncertainty = wm.uncertainty(&query);
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
''')
print("[1] Planner lib.rs teljesen újraírva.")

# 2. Tesztek frissítése
write_file("crates/athlesia-planner/tests/planner_full_test.rs", r'''
use athlesia_planner::{Planner, PlannerMode};
use athlesia_world_model::WorldModel;
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
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
''')
print("[2] Planner tesztek hozzáadva.")

# 3. Tesztek futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-planner", "--test", "planner_full_test"], capture_output=True, text=True, check=False)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Planner tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Planner tesztek zöldek.")

# 4. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Finalize Planner with Search Engine integration"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
