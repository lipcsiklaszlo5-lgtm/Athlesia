
use athlesia_world_model::{WorldModel, KnowledgeState, Prediction, Observation};
use athlesia_abstraction::AbstractionEngine;
use athlesia_knowledge::KnowledgeBase;
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

    /// Az open-world ciklus kimenettel együtt.
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
        let (state, residual) = wm.evaluate_with_residual(action, prediction, observation);
        if state != KnowledgeState::OutOfModel {
            return OpenWorldOutcome::NotOutOfModel;
        }

        let residuals = vec![residual];
        let candidate = match AbstractionEngine::discover_candidate_concept(&residuals) {
            Some(c) => c,
            None => return OpenWorldOutcome::Abstain,
        };

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
            OpenWorldOutcome::Abstain
        }
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
        let (state, residual) = wm.evaluate_with_residual(action, prediction, observation);
        if state != KnowledgeState::OutOfModel {
            return None;
        }

        let residuals = vec![residual];
        let candidate = AbstractionEngine::discover_candidate_concept(&residuals)?;

        // Transfer: ha már van ilyen kapcsolati mintánk, használjuk azt.
        if let Some(existing) = kb
            .get_verified_concepts()
            .iter()
            .find(|c| c.relation_pattern == candidate.sketch.relation_pattern)
        {
            return Some(existing.clone());
        }

        // Egyszerű verifikáció: ha a candidate confidence elér egy küszöböt,
        // igazolt fogalomként tároljuk.
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
