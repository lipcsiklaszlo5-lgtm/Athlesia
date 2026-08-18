
use athlesia_world_model::{WorldModel, KnowledgeState, Prediction, Observation};
use athlesia_abstraction::AbstractionEngine;
use athlesia_knowledge::KnowledgeBase;
use athlesia_types::Action;

/// Open-world ciklus: reziduálisból fogalomjelölt, majd egyszerű verifikáció.
pub struct OpenWorldCycle;

impl OpenWorldCycle {
    /// Lefuttatja a Phase 13 alapciklust.
    ///
    /// 1. Kiértékeli a predikciót (tudásállapot + reziduális).
    /// 2. Ha OutOfModel, a reziduálisból candidate conceptet generál.
    /// 3. Ha a candidate confidence elég magas, verifikálja és beilleszti a KB-be.
    pub fn run(
        wm: &WorldModel,
        action: &Action,
        prediction: &Prediction,
        observation: &Observation,
        kb: &mut KnowledgeBase,
    ) -> Option<athlesia_knowledge::VerifiedConcept> {
        let (state, residual) = wm.evaluate_with_residual(action, prediction, observation);
        if state != KnowledgeState::OutOfModel {
            return None;
        }

        let residuals = vec![residual];
        let candidate = AbstractionEngine::discover_candidate_concept(&residuals)?;

        // Egyszerű verifikáció: ha a candidate confidence elér egy küszöböt,
        // igazolt fogalomként kezeljük.
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
            Some(verified)
        } else {
            None
        }
    }
}
