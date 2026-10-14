//! Bounded validity ledger for *M47-derived* predictions, owned by M51.
//! Not an action selector, second world model, ARC adapter or global permit gate.
//! Issue a witness BEFORE acting; resolve it BEFORE admitting new learning evidence.
use super::{CognitiveStructure, OnlinePersistentCognitiveState};
use athlesia_universal_domain_learning::{
    GroundedExecutableWorldModelPolicy, GroundedStateSnapshot, GroundedStructuralPrediction,
    TransitionSchemaPolicy,
};

const MAX_CAUSAL_OBSERVATIONS: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalPredictionWitness {
    source: GroundedStateSnapshot,
    prediction: GroundedStructuralPrediction,
    episode_epoch: usize,
}

impl CausalPredictionWitness {
    pub fn source(&self) -> &GroundedStateSnapshot {
        &self.source
    }
    pub fn action(&self) -> &CognitiveStructure {
        self.prediction.transformation()
    }
    pub fn prediction(&self) -> &GroundedStructuralPrediction {
        &self.prediction
    }
    pub fn episode_epoch(&self) -> usize {
        self.episode_epoch
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalObservationStatus {
    ConfirmedProvisional,
    ContradictedQuarantined,
    ExactDuplicate,
    ConflictingEventIdentity,
    EvidenceFrontierExceeded,
    StaleEvidenceEpoch,
    MissingWitness,
    WitnessMismatchQuarantined,
    IndeterminateDispatchQuarantined,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalCounterexample {
    event_index: u64,
    witness: CausalPredictionWitness,
    observed: GroundedStateSnapshot,
}

impl CausalCounterexample {
    pub fn event_index(&self) -> u64 {
        self.event_index
    }
    pub fn witness(&self) -> &CausalPredictionWitness {
        &self.witness
    }
    pub fn observed(&self) -> &GroundedStateSnapshot {
        &self.observed
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CausalObservedEvent {
    event_index: u64,
    witness: CausalPredictionWitness,
    observed: GroundedStateSnapshot,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct CausalValidityLedger {
    pending: Option<CausalPredictionWitness>,
    events: Vec<CausalObservedEvent>,
    counterexamples: Vec<CausalCounterexample>,
    quarantined: Vec<(GroundedStateSnapshot, CognitiveStructure)>,
    integrity_compromised: bool,
}

impl CausalValidityLedger {
    pub(super) fn pending_witness(&self) -> Option<CausalPredictionWitness> {
        self.pending.clone()
    }

    pub(super) fn blocked(
        &self,
        source: &GroundedStateSnapshot,
        action: &CognitiveStructure,
    ) -> bool {
        self.integrity_compromised
            || self
                .quarantined
                .iter()
                .any(|(s, a)| s == source && a == action)
    }
    fn quarantine(&mut self, witness: &CausalPredictionWitness) {
        let key = (witness.source.clone(), witness.action().clone());
        if !self.quarantined.contains(&key) {
            self.quarantined.push(key);
        }
    }
}

impl OnlinePersistentCognitiveState {
    /// Produce a one-shot, epoch-bound witness for an empirically supported M47 prediction.
    /// This is NOT M48 authorization to dispatch an action.
    pub fn begin_causal_prediction_observation(
        &mut self,
        state: &GroundedStateSnapshot,
        action: &CognitiveStructure,
        schema_policy: TransitionSchemaPolicy,
        model_policy: GroundedExecutableWorldModelPolicy,
    ) -> Option<CausalPredictionWitness> {
        if self.causal_validity.pending.is_some() || self.causal_validity.blocked(state, action) {
            return None;
        }
        let authority = self.current_empirically_authorized_structural_prediction(
            state,
            action,
            schema_policy,
            model_policy,
        )?;
        let witness = CausalPredictionWitness {
            source: state.clone(),
            prediction: authority.prediction().clone(),
            episode_epoch: self.transition_episode_count(),
        };
        self.causal_validity.pending = Some(witness.clone());
        Some(witness)
    }

    /// Record independently observed post-action state (not a caller-reported success flag).
    /// Exact diff required; unmodelled side effects contradict the issued prediction.
    pub fn complete_causal_prediction_observation(
        &mut self,
        witness: &CausalPredictionWitness,
        event_index: u64,
        observed: &GroundedStateSnapshot,
    ) -> CausalObservationStatus {
        let ledger = &mut self.causal_validity;
        if let Some(previous) = ledger.events.iter().find(|e| e.event_index == event_index) {
            if &previous.witness == witness && &previous.observed == observed {
                return CausalObservationStatus::ExactDuplicate;
            }
            ledger.integrity_compromised = true;
            ledger.quarantine(witness);
            ledger.pending = None;
            return CausalObservationStatus::ConflictingEventIdentity;
        }
        if ledger.pending.is_none() {
            return CausalObservationStatus::MissingWitness;
        }
        if ledger.pending.as_ref() != Some(witness) {
            ledger.integrity_compromised = true;
            ledger.quarantine(witness);
            ledger.pending = None;
            return CausalObservationStatus::WitnessMismatchQuarantined;
        }
        ledger.pending = None;
        if witness.episode_epoch != self.transition_schema_learning.episode_count() {
            ledger.integrity_compromised = true;
            ledger.quarantine(witness);
            return CausalObservationStatus::StaleEvidenceEpoch;
        }
        if ledger
            .events
            .last()
            .is_some_and(|e| event_index <= e.event_index)
            || ledger.events.len() >= MAX_CAUSAL_OBSERVATIONS
        {
            ledger.integrity_compromised = true;
            ledger.quarantine(witness);
            return CausalObservationStatus::EvidenceFrontierExceeded;
        }
        let additions: Vec<_> = observed
            .facts()
            .iter()
            .filter(|fact| !witness.source.contains_fact(fact))
            .cloned()
            .collect();
        let removals: Vec<_> = witness
            .source
            .facts()
            .iter()
            .filter(|fact| !observed.contains_fact(fact))
            .cloned()
            .collect();
        let contradicted = additions.as_slice() != witness.prediction.additions()
            || removals.as_slice() != witness.prediction.removals();
        ledger.events.push(CausalObservedEvent {
            event_index,
            witness: witness.clone(),
            observed: observed.clone(),
        });
        if contradicted {
            ledger.counterexamples.push(CausalCounterexample {
                event_index,
                witness: witness.clone(),
                observed: observed.clone(),
            });
            ledger.quarantine(witness);
            CausalObservationStatus::ContradictedQuarantined
        } else {
            CausalObservationStatus::ConfirmedProvisional
        }
    }

    /// Unknown transport outcome must never be recorded as a successful transition.
    pub fn quarantine_indeterminate_causal_dispatch(
        &mut self,
        witness: &CausalPredictionWitness,
    ) -> CausalObservationStatus {
        let ledger = &mut self.causal_validity;
        if ledger.pending.as_ref() != Some(witness) {
            ledger.integrity_compromised = true;
            return CausalObservationStatus::WitnessMismatchQuarantined;
        }
        ledger.pending = None;
        ledger.quarantine(witness);
        CausalObservationStatus::IndeterminateDispatchQuarantined
    }

    /// The transport boundary can inspect whether a pre-dispatch M47 witness
    /// still awaits an independent observation. This is not action authority.
    pub fn pending_causal_prediction_observation(&self) -> Option<&CausalPredictionWitness> {
        self.causal_validity.pending.as_ref()
    }

    /// A lost/ambiguous transport result or ungroundable successor must never
    /// leave an old witness pending or fabricate a positive observation.
    pub fn quarantine_pending_causal_dispatch(&mut self) -> Option<CausalObservationStatus> {
        let witness = self.causal_validity.pending_witness()?;
        Some(self.quarantine_indeterminate_causal_dispatch(&witness))
    }

    pub fn causal_observation_count(&self) -> usize {
        self.causal_validity.events.len()
    }
    pub fn causal_counterexamples(&self) -> &[CausalCounterexample] {
        &self.causal_validity.counterexamples
    }
    pub fn causal_quarantine_count(&self) -> usize {
        self.causal_validity.quarantined.len()
    }
    pub fn causal_integrity_compromised(&self) -> bool {
        self.causal_validity.integrity_compromised
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use athlesia_mindstone_sparse_cognition::CognitiveSignal;
    use athlesia_universal_domain_learning::GroundedTransformationEpisode;

    fn a(n: u64) -> CognitiveStructure {
        CognitiveStructure::atom(n)
    }
    fn state(values: &[u64]) -> GroundedStateSnapshot {
        GroundedStateSnapshot::new(values.iter().copied().map(a).collect()).unwrap()
    }
    fn episode(before: &[u64], after: &[u64], action: u64) -> GroundedTransformationEpisode {
        GroundedTransformationEpisode::new(state(before), state(after), a(action))
    }
    fn policies() -> (TransitionSchemaPolicy, GroundedExecutableWorldModelPolicy) {
        (
            TransitionSchemaPolicy::new(
                2,
                CognitiveSignal::new(1).unwrap(),
                CognitiveSignal::new(1).unwrap(),
                64,
                64,
            )
            .unwrap(),
            GroundedExecutableWorldModelPolicy::new(64).unwrap(),
        )
    }
    fn trained() -> OnlinePersistentCognitiveState {
        let mut owner = OnlinePersistentCognitiveState::new();
        owner.transition_schema_learning.episodes = vec![
            episode(&[1], &[1, 900], 10),
            episode(&[2], &[2, 900], 10),
            episode(&[3], &[3], 20),
            episode(&[4], &[4], 20),
        ];
        owner
    }
    fn issued(owner: &mut OnlinePersistentCognitiveState) -> CausalPredictionWitness {
        let (s, m) = policies();
        owner.begin_causal_prediction_observation(&state(&[1]), &a(10), s, m)
            .expect("independent positive and negative action evidence should induce a predictive model")
    }
    #[test]
    fn r3b_missing_transport_outcome_consumes_pending_without_fabricating_success() {
        let mut owner = trained();
        let witness = issued(&mut owner);
        assert_eq!(
            owner.pending_causal_prediction_observation(),
            Some(&witness)
        );
        assert_eq!(
            owner.quarantine_pending_causal_dispatch(),
            Some(CausalObservationStatus::IndeterminateDispatchQuarantined)
        );
        assert!(owner.pending_causal_prediction_observation().is_none());
        assert_eq!(owner.causal_observation_count(), 0);
        assert_eq!(owner.causal_quarantine_count(), 1);
        assert!(owner.quarantine_pending_causal_dispatch().is_none());
        let (schema, model) = policies();
        assert!(owner
            .current_empirically_authorized_structural_prediction(
                witness.source(),
                witness.action(),
                schema,
                model,
            )
            .is_none());
    }

    #[test]
    fn cold_knowledge_must_not_issue_a_prediction_witness() {
        let mut owner = OnlinePersistentCognitiveState::new();
        let (schema, model) = policies();
        assert!(owner
            .begin_causal_prediction_observation(&state(&[1]), &a(10), schema, model,)
            .is_none());
        assert_eq!(owner.causal_observation_count(), 0);
    }
    #[test]
    fn a_second_witness_cannot_replace_an_unresolved_action() {
        let mut owner = trained();
        let ticket = issued(&mut owner);
        let (schema, model) = policies();
        assert!(owner
            .begin_causal_prediction_observation(&state(&[2]), &a(10), schema, model,)
            .is_none());
        assert_eq!(
            owner.quarantine_indeterminate_causal_dispatch(&ticket),
            CausalObservationStatus::IndeterminateDispatchQuarantined
        );
    }
    #[test]
    fn correct_effect_is_provisional_and_duplicate_is_idempotent() {
        let mut owner = trained();
        let ticket = issued(&mut owner);
        assert_eq!(
            owner.complete_causal_prediction_observation(&ticket, 10, &state(&[1, 900])),
            CausalObservationStatus::ConfirmedProvisional
        );
        assert_eq!(
            owner.complete_causal_prediction_observation(&ticket, 10, &state(&[1, 900])),
            CausalObservationStatus::ExactDuplicate
        );
        assert_eq!(owner.causal_observation_count(), 1);
        assert!(owner.causal_counterexamples().is_empty());
    }
    #[test]
    fn contradiction_quarantines_future_model_based_prediction() {
        let mut owner = trained();
        let ticket = issued(&mut owner);
        assert_eq!(
            owner.complete_causal_prediction_observation(&ticket, 11, &state(&[1])),
            CausalObservationStatus::ContradictedQuarantined
        );
        assert_eq!(owner.causal_counterexamples().len(), 1);
        assert_eq!(owner.causal_quarantine_count(), 1);
        let (s, m) = policies();
        assert!(owner
            .current_empirically_authorized_structural_prediction(&state(&[1]), &a(10), s, m)
            .is_none());
        assert!(owner
            .begin_causal_prediction_observation(&state(&[1]), &a(10), s, m)
            .is_none());
    }
    #[test]
    fn indeterminate_dispatch_does_not_fabricate_a_success() {
        let mut owner = trained();
        let ticket = issued(&mut owner);
        assert_eq!(
            owner.quarantine_indeterminate_causal_dispatch(&ticket),
            CausalObservationStatus::IndeterminateDispatchQuarantined
        );
        assert_eq!(owner.causal_observation_count(), 0);
        assert_eq!(owner.causal_quarantine_count(), 1);
    }
    #[test]
    fn stale_epoch_cannot_launder_old_prediction() {
        let mut owner = trained();
        let ticket = issued(&mut owner);
        owner
            .transition_schema_learning
            .episodes
            .push(episode(&[7], &[7, 900], 10));
        assert_eq!(
            owner.complete_causal_prediction_observation(&ticket, 12, &state(&[1, 900])),
            CausalObservationStatus::StaleEvidenceEpoch
        );
        assert!(owner.causal_integrity_compromised());
    }
}
