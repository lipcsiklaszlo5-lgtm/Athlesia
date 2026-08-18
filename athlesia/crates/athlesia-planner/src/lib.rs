
use athlesia_types::{Grid, PrimName, Params, Program, Action};
use athlesia_search::{SearchEngine, DefaultSearchEngine, SearchStrategy};
use athlesia_world_model::{WorldModel, Query};
use athlesia_hypothesis::CandidateConcept;

/// A tervező üzemmódja.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlannerMode {
    GoalDirected,
    Exploration,
}

/// Egy akció értékelése a döntési ciklushoz.
/// A mezők súlyozva kombinálhatók a `value` függvényben.
#[derive(Debug, Clone)]
pub struct ActionValue {
    pub expected_information_gain: f32,
    pub expected_progress: f32,
    pub action_cost: f32,
    pub risk: f32,
}


/// Kísérleti terv a candidate concept aktív verifikálásához.
#[derive(Debug, Clone)]
pub struct ExperimentPlan {
    pub actions: Vec<athlesia_types::Action>,
    pub target_hypothesis: String,
    pub expected_observation: String,
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


impl Planner {
    /// Kiszámítja egy akció ActionValue értékelését.
    ///
    /// - `expected_information_gain`: a predikció bizonytalansága (1 - confidence).
    /// - `expected_progress`: ha van cél, a pixel-egyezés javulása az akció után.
    /// - `action_cost`: egyszerűsített költség (jelenleg konstans 1).
    /// - `risk`: jelenleg 0, később bővíthető.
    pub fn compute_action_value(
        &self,
        current: &Grid,
        target: Option<&Grid>,
        action: &Action,
        wm: &WorldModel,
    ) -> ActionValue {
        let query = Query {
            state: current.clone(),
            action: action.clone(),
        };
        let prediction = wm.predict(&query.state, &query.action);
        let uncertainty = 1.0 - prediction.confidence as f32;

        let info_gain = uncertainty;

        let progress = if let Some(target_grid) = target {
            let before = pixel_match(current, target_grid);
            let after = pixel_match(&prediction.state, target_grid);
            (after as f32 - before as f32) // lehet negatív is
        } else {
            0.0
        };

        ActionValue {
            expected_information_gain: info_gain,
            expected_progress: progress,
            action_cost: 1.0,
            risk: 0.0,
        }
    }

    /// Kísérleti tervet készít egy candidate concept alapján.
    /// Jelenleg egyszerű placeholder: az akciósor üres, de a célhipotézis
    /// neve rögzítve van. A következő mikrolépésekben lesz diszkriminatív.
    pub fn plan_experiment(&self, candidate: &CandidateConcept) -> ExperimentPlan {
        // A kísérleti akciót a jelenlegi heurisztika alapján választjuk.
        let probe_action = self.select_probe_action(candidate);
        ExperimentPlan {
            actions: vec![probe_action],
            target_hypothesis: candidate.sketch.name.clone(),
            expected_observation: candidate.sketch.relation_pattern.clone(),
        }
    }


    /// Egyetlen egyszerű kísérleti akciót választ a candidate concept alapján.
    ///
    /// Jelenleg csak a relation_pattern stringjére hagyatkozik:
    /// - ha tartalmazza az "interaction" szót, Translate(1,0)
    /// - ha tartalmazza a "symmetry" szót, ReflectH
    /// - különben Translate(0,1)
    ///
    /// Ez egy placeholder heurisztika, amit később információnyerés-alapú
    /// diszkriminatív akcióválasztással váltunk ki.
    pub fn select_probe_action(&self, candidate: &CandidateConcept) -> Action {
        let pattern = candidate.sketch.relation_pattern.to_lowercase();
        if pattern.contains("interaction") {
            Action { prim: PrimName::Translate, params: Params::Translate(1, 0) }
        } else if pattern.contains("symmetry") {
            Action { prim: PrimName::ReflectH, params: Params::None }
        } else {
            Action { prim: PrimName::Translate, params: Params::Translate(0, 1) }
        }
    }

    /// Kiválasztja a legjobb akciót a megadott súlyokkal.
    ///
    /// `value = α * info_gain + β * progress - γ * cost - δ * risk`
    pub fn select_action(
        &self,
        current: &Grid,
        target: Option<&Grid>,
        actions: &[Action],
        wm: &WorldModel,
        alpha: f32,
        beta: f32,
        gamma: f32,
        delta: f32,
    ) -> Option<Action> {
        let mut best: Option<(Action, f32)> = None;

        for action in actions {
            let av = self.compute_action_value(current, target, action, wm);
            let total = alpha * av.expected_information_gain
                + beta * av.expected_progress
                - gamma * av.action_cost
                - delta * av.risk;

            if best.is_none() || total > best.as_ref().unwrap().1 {
                best = Some((action.clone(), total));
            }
        }

        best.map(|(a, _)| a)
    }
}

/// Két grid pixel-egyezésének aránya 0.0 és 1.0 között.
fn pixel_match(a: &Grid, b: &Grid) -> f32 {
    if a.width != b.width || a.height != b.height {
        return 0.0;
    }
    let total = (a.width as usize) * (a.height as usize);
    if total == 0 {
        return 1.0;
    }
    let mut matching = 0usize;
    for i in 0..a.height as usize {
        for j in 0..a.width as usize {
            let idx = i * a.width as usize + j;
            let bidx = i * b.width as usize + j;
            if a.cells[idx] == b.cells[bidx] {
                matching += 1;
            }
        }
    }
    matching as f32 / total as f32
}
