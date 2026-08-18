
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
                        action: action.clone(),
                    };
                    let uncertainty = wm.uncertainty(&query);
                    if uncertainty > max_uncertainty {
                        max_uncertainty = uncertainty;
                        best_action = Some(action.clone());
                    }
                }

                best_action.map(|a| vec![(a.prim, a.params)])
            }
        }
    }
}
