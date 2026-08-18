
use athlesia_world_model::{WorldModel, KnowledgeState, Prediction, Observation};
use athlesia_abstraction::AbstractionEngine;
use athlesia_knowledge::KnowledgeBase;
use athlesia_metalearner::MetaLearner;
use athlesia_planner::{Planner, PlannerMode, ExperimentRequest};
use athlesia_types::Action;

/// Open-world ciklus: reziduálisból fogalomjelölt, majd egyszerű verifikáció.
pub struct OpenWorldCycle;

/// Az open-world ciklus kimenete.
#[derive(Debug, Clone, PartialEq)]
pub enum OpenWorldOutcome {
    NotOutOfModel,
    Abstain,
    Retrieved(athlesia_knowledge::VerifiedConcept),
    Verified(athlesia_knowledge::VerifiedConcept),
}

impl OpenWorldCycle {
    /// Open-world ciklus MetaLearner integrációval.
    ///
    /// Ha a candidate relation_pattern szerepel a MetaLearner
    /// `failed_concepts` archívumában, azonnal Abstain.
    /// Ha a candidate confidence < 0.5, rögzíti a kudarcot.
    pub fn run_with_meta(
        wm: &WorldModel,
        action: &Action,
        prediction: &Prediction,
        observation: &Observation,
        kb: &mut KnowledgeBase,
        meta: &mut MetaLearner,
    ) -> OpenWorldOutcome {
        let (state, residual) = wm.evaluate_with_residual(action, prediction, observation);
        if state != KnowledgeState::OutOfModel {
            return OpenWorldOutcome::NotOutOfModel;
        }

        let residuals = vec![residual];
        let candidate = match AbstractionEngine::discover_candidate_concept(&residuals) {
            Some(c) => c,
            None => return OpenWorldOutcome::Abstain,
        };

        // Ismert kudarc ellenőrzése a MetaLearner archívumban.
        if meta.is_known_failed_concept(&candidate.sketch.relation_pattern) {
            return OpenWorldOutcome::Abstain;
        }

        if let Some(existing) = kb
            .get_verified_concepts()
            .iter()
            .find(|c| c.relation_pattern == candidate.sketch.relation_pattern)
        {
            return OpenWorldOutcome::Retrieved(existing.clone());
        }

        if candidate.confidence >= 0.5 {
            let verified = athlesia_knowledge::VerifiedConcept {
                id: kb.get_verified_concepts().len() as u64,
                name: candidate.sketch.name.clone(),
                relation_pattern: candidate.sketch.relation_pattern.clone(),
                evidence_count: candidate.evidence.len(),
            };
            kb.add_verified_concept(
                verified.name.clone(),
                verified.relation_pattern.clone(),
                verified.evidence_count,
            );
            OpenWorldOutcome::Verified(verified)
        } else {
            // Kudarc rögzítése a MetaLearner archívumban.
            meta.record_failed_concept(candidate.sketch.relation_pattern.clone());
            OpenWorldOutcome::Abstain
        }
    }

    /// Kísérleti kérést generál a reziduálisból felfedezett candidate concepthez.
    ///
    /// Ha a rendszer OutOfModel, és a candidate confidence elég magas
    /// (>= 0.5), akkor kísérleti kérést ad; különben `None`.
    pub fn prepare_experiment(
        wm: &WorldModel,
        action: &Action,
        prediction: &Prediction,
        observation: &Observation,
        _kb: &KnowledgeBase,
    ) -> Option<athlesia_planner::ExperimentRequest> {
        let (state, residual) = wm.evaluate_with_residual(action, prediction, observation);
        if state != KnowledgeState::OutOfModel {
            return None;
        }

        let residuals = vec![residual];
        let candidate = AbstractionEngine::discover_candidate_concept(&residuals)?;

        if candidate.confidence < 0.5 {
            return None;
        }

        let planner = Planner::new(PlannerMode::Exploration);
        Some(planner.plan_experiment_request(&candidate))
    }


    /// Generikus kísérleti ciklus.
    ///
    /// A megadott `execute` closure végrehajtja az `ExperimentRequest` akcióját,
    /// és visszaadja a megfigyelt állapotot. Ezután az OpenWorldCycle
    /// lefuttatja a fogalomtanulást a predikció és a megfigyelés alapján.
    ///
    /// F: FnMut(&Action) -> Observation
    pub fn run_experiment_cycle<F>(
        wm: &WorldModel,
        kb: &mut KnowledgeBase,
        meta: &mut MetaLearner,
        request: ExperimentRequest,
        mut execute: F,
    ) -> OpenWorldOutcome
    where
        F: FnMut(&Action) -> Observation,
    {
        // Predikció készítése a jelenlegi állapotból.
        let current_state = wm.current_state.clone();
        let prediction = wm.predict(&current_state, &request.action);

        // Az akció végrehajtása a külvilág által.
        let observation = execute(&request.action);

        // OpenWorld futtatása a megfigyeléssel.
        Self::run_with_meta(wm, &request.action, &prediction, &observation, kb, meta)
    }

    /// Az open-world ciklus kimenettel együtt (MetaLearner nélkül).
    ///
    /// - Ha nem OutOfModel: NotOutOfModel
    /// - Ha van OutOfModel, de a candidate confidence < 0.5: Abstain
    /// - Ha a relation_pattern már létezik: Retrieved
    /// - Különben Verified
    pub fn run_with_outcome(
        wm: &WorldModel,
        action: &Action,
        prediction: &Prediction,
        observation: &Observation,
        kb: &mut KnowledgeBase,
    ) -> OpenWorldOutcome {
        let mut meta = MetaLearner::new();
        Self::run_with_meta(wm, action, prediction, observation, kb, &mut meta)
    }

    /// Lefuttatja a Phase 13 alapciklust.
    ///
    /// 1. Kiértékeli a predikciót (tudásállapot + reziduális).
    /// 2. Ha OutOfModel, a reziduálisból candidate conceptet generál.
    /// 3. Ha a candidate relation_pattern már létezik a KB-ben, visszaadja azt.
    /// 4. Különben, ha a candidate confidence elég magas, verifikálja és beilleszti a KB-be.
    pub fn run(
        wm: &WorldModel,
        action: &Action,
        prediction: &Prediction,
        observation: &Observation,
        kb: &mut KnowledgeBase,
    ) -> Option<athlesia_knowledge::VerifiedConcept> {
        let mut meta = MetaLearner::new();
        match Self::run_with_meta(wm, action, prediction, observation, kb, &mut meta) {
            OpenWorldOutcome::Verified(v) | OpenWorldOutcome::Retrieved(v) => Some(v),
            _ => None,
        }
    }
}
