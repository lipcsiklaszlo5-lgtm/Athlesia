
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
