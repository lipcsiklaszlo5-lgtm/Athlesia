
use athlesia_types::{Grid, PrimName, Params, Program};
use athlesia_search::search;

/// A tervező üzemmódja.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlannerMode {
    GoalDirected,
    Exploration,
}

/// A Manhattan Kernel tervezője.
///
/// - Cél-irányított mód: egy ismert cél-grid eléréséhez készít programot.
///   Ez a legegyszerűbb, de hasznos proxy a valódi cél-irányított tervezésre.
/// - Feltáró mód: mivel még nincs cél, egy alapértelmezett, determinisztikus
///   akciót javasol, amely a későbbiekben információnyereség-alapúvá bővülhet.
#[derive(Debug)]
pub struct Planner {
    pub mode: PlannerMode,
}

impl Planner {
    pub fn new(mode: PlannerMode) -> Self {
        Planner { mode }
    }

    /// Terv készítése. Ha a cél ismert, és elérhető max_depth lépésen belül,
    /// a Search Engine segítségével visszaad egy programszekvenciát.
    pub fn plan(&self, current: &Grid, target: Option<&Grid>, max_depth: usize) -> Option<Program> {
        match self.mode {
            PlannerMode::GoalDirected => {
                // Ha nincs cél, nincs cél-irányított terv
                target?;
                search(current, target.unwrap(), max_depth)
            }
            PlannerMode::Exploration => {
                // Determinisztikus feltáró akció: egy lépés jobbra.
                // A későbbi változatban ez bizonytalanság-alapú lesz.
                Some(vec![(PrimName::Translate, Params::Translate(1, 0))])
            }
        }
    }
}
