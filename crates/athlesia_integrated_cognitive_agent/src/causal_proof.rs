//! Fail-closed execution proof boundary for frozen causal transfer.
//!
//! This module sits above the existing CausalLearner. It does not change how
//! hypotheses are learned. It makes the execution boundary explicit:
//! a model is cloned, its candidate set is bounded, a prediction is committed
//! against a scope and pre-state, a host permit is consumed, and only then may
//! the outcome be resolved.
//!
//! The digest used here is the repository's existing deterministic u64 digest.
//! It is an integrity handle for local replay, not a cryptographic signature.
//! Cryptographic Causal Knowledge Sync belongs to the later P4 protocol.

use super::contextual_causal_agent::ContextId;
use super::cross_domain_causal_core::{
    ActionId, CausalLearner, Digest, State,
};

const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;
const COMMITMENT_DOMAIN: u64 = 0x4154_4c45_5349_4150;

fn mix(mut state: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        state ^= u64::from(byte);
        state = state.wrapping_mul(FNV_PRIME);
    }
    state
}

/// Identity of the prediction request and the pre-commitment epoch.
///
/// A result from one scope cannot be replayed in another scope. The schema
/// and abstraction digests are supplied by the caller so continuous input
/// quantization can be bound to the same proof as the causal model.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PredictionScope {
    pub schema_digest: Digest,
    pub abstraction_digest: Digest,
    pub context: Option<ContextId>,
    pub epoch: u64,
    pub episode: u64,
    pub sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FrozenPredictionError {
    EmptyModel,
    EmptyOutcomeSet,
    QuarantinedModel,
    UnknownAction,
    CandidateBudgetExceeded { observed: usize, limit: usize },
    OutcomeBudgetExceeded { observed: usize, limit: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProofError {
    Prediction(FrozenPredictionError),
    StaleScope,
    WrongCommand,
    WrongAction,
    BeforeStateMismatch,
    PermitNotConsumed,
    PermitAlreadyConsumed,
    CommitmentAlreadyResolved,
}

impl From<FrozenPredictionError> for ProofError {
    fn from(error: FrozenPredictionError) -> Self {
        Self::Prediction(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitmentResult {
    Confirmed,
    Contradicted,
    Indeterminate,
}

/// Immutable execution view of a learner.
///
/// The learner is cloned at construction. Later online revision of the
/// original learner cannot mutate this view, which makes the candidate set
/// used for a proof auditable and replayable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrozenTransferModel {
    learner: CausalLearner,
    model_digest: Digest,
    max_candidates: usize,
    max_outcomes: usize,
}

impl FrozenTransferModel {
    pub fn from_learner(
        learner: &CausalLearner,
        max_candidates: usize,
        max_outcomes: usize,
    ) -> Result<Self, FrozenPredictionError> {
        if learner.quarantined() {
            return Err(FrozenPredictionError::QuarantinedModel);
        }
        if learner.version_space().is_empty() {
            return Err(FrozenPredictionError::EmptyModel);
        }
        if learner.version_space().len() > max_candidates {
            return Err(FrozenPredictionError::CandidateBudgetExceeded {
                observed: learner.version_space().len(),
                limit: max_candidates,
            });
        }
        Ok(Self {
            learner: learner.clone(),
            model_digest: learner.model_digest(),
            max_candidates,
            max_outcomes,
        })
    }

    pub fn model_digest(&self) -> Digest { self.model_digest }
    pub fn candidate_count(&self) -> usize { self.learner.version_space().len() }
    pub fn max_candidates(&self) -> usize { self.max_candidates }
    pub fn max_outcomes(&self) -> usize { self.max_outcomes }

    /// True when the source learner still has the exact frozen model digest.
    pub fn is_unchanged(&self, learner: &CausalLearner) -> bool {
        self.model_digest == learner.model_digest()
            && !learner.quarantined()
            && learner.version_space() == self.learner.version_space()
    }

    /// Enumerate all distinct effects permitted by the frozen candidate set.
    ///
    /// Any unknown action or outcome explosion rejects the prediction. A
    /// caller must handle the error as ABSTAIN; there is no best-effort
    /// fallback.
    pub fn predict(
        &self,
        action: ActionId,
        before: &State,
    ) -> Result<Vec<State>, FrozenPredictionError> {
        if self.learner.version_space().is_empty() {
            return Err(FrozenPredictionError::EmptyModel);
        }
        if self.learner.version_space().len() > self.max_candidates {
            return Err(FrozenPredictionError::CandidateBudgetExceeded {
                observed: self.learner.version_space().len(),
                limit: self.max_candidates,
            });
        }
        if self
            .learner
            .version_space()
            .iter()
            .any(|candidate| candidate.action != action)
        {
            return Err(FrozenPredictionError::UnknownAction);
        }

        let mut outcomes = Vec::new();
        for candidate in self.learner.version_space() {
            let outcome = candidate.predicts(before);
            if !outcomes.contains(&outcome) {
                outcomes.push(outcome);
                if outcomes.len() > self.max_outcomes {
                    return Err(FrozenPredictionError::OutcomeBudgetExceeded {
                        observed: outcomes.len(),
                        limit: self.max_outcomes,
                    });
                }
            }
        }
        outcomes.sort_by_key(|state| state.digest());
        Ok(outcomes)
    }

    /// Bind a bounded prediction to a scope, command, action, and exact state.
    pub fn commit(
        &self,
        scope: PredictionScope,
        command: u64,
        action: ActionId,
        before: &State,
    ) -> Result<PredictionCommitment, FrozenPredictionError> {
        let possible_after = self.predict(action, before)?;
        PredictionCommitment::from_bounded_outcomes(
            scope,
            command,
            action,
            self.model_digest,
            before,
            possible_after,
            self.max_outcomes,
        )
    }
}

/// A one-shot prediction record.
///
/// The possible outcome set is retained instead of selecting a single
/// statistical guess. This is the point at which the kernel can safely
/// choose ABSTAIN whenever the set cannot be represented within budget.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredictionCommitment {
    scope: PredictionScope,
    command: u64,
    action: ActionId,
    before_digest: Digest,
    possible_after: Vec<State>,
    model_digest: Digest,
    commitment_digest: Digest,
    resolved: bool,
}

impl PredictionCommitment {
    pub(crate) fn from_bounded_outcomes(
        scope: PredictionScope,
        command: u64,
        action: ActionId,
        model_digest: Digest,
        before: &State,
        mut possible_after: Vec<State>,
        max_outcomes: usize,
    ) -> Result<Self, FrozenPredictionError> {
        if possible_after.is_empty() {
            return Err(FrozenPredictionError::EmptyOutcomeSet);
        }
        if possible_after.len() > max_outcomes {
            return Err(FrozenPredictionError::OutcomeBudgetExceeded {
                observed: possible_after.len(),
                limit: max_outcomes,
            });
        }
        possible_after.sort_by_key(|state| state.digest());
        possible_after.dedup();
        let commitment_digest = commitment_digest(
            &scope,
            model_digest,
            command,
            action,
            before.digest(),
            &possible_after,
        );
        Ok(Self {
            scope,
            command,
            action,
            before_digest: before.digest(),
            possible_after,
            model_digest,
            commitment_digest,
            resolved: false,
        })
    }

    pub fn scope(&self) -> PredictionScope { self.scope }
    pub fn command(&self) -> u64 { self.command }
    pub fn action(&self) -> ActionId { self.action }
    pub fn before_digest(&self) -> Digest { self.before_digest }
    pub fn possible_after(&self) -> &[State] { &self.possible_after }
    pub fn model_digest(&self) -> Digest { self.model_digest }
    pub fn commitment_digest(&self) -> Digest { self.commitment_digest }
    pub fn is_resolved(&self) -> bool { self.resolved }

    /// Issue a host-bound, one-use permit after policy checks.
    pub fn issue_permit(
        &self,
        host_policy_digest: Digest,
        nonce: u64,
    ) -> ExecutionPermit {
        ExecutionPermit {
            commitment_digest: self.commitment_digest,
            scope: self.scope,
            command: self.command,
            action: self.action,
            before_digest: self.before_digest,
            host_policy_digest,
            nonce,
            consumed: false,
        }
    }

    /// Consume a permit only when its binding is exact.
    pub fn consume_permit(
        &mut self,
        permit: &mut ExecutionPermit,
        scope: PredictionScope,
        command: u64,
        action: ActionId,
        before: &State,
    ) -> Result<(), ProofError> {
        self.check_binding(scope, command, action, before)?;
        if permit.consumed {
            return Err(ProofError::PermitAlreadyConsumed);
        }
        if permit.commitment_digest != self.commitment_digest
            || permit.scope != scope
            || permit.command != command
            || permit.action != action
            || permit.before_digest != before.digest()
        {
            return Err(ProofError::StaleScope);
        }
        permit.consumed = true;
        Ok(())
    }

    /// Resolve an executed result against the committed outcome set.
    pub fn resolve(
        &mut self,
        permit: &ExecutionPermit,
        scope: PredictionScope,
        command: u64,
        action: ActionId,
        before: &State,
        after: &State,
    ) -> Result<CommitmentResult, ProofError> {
        self.check_binding(scope, command, action, before)?;
        self.check_permit(permit)?;
        if self.resolved {
            return Err(ProofError::CommitmentAlreadyResolved);
        }
        self.resolved = true;
        if self.possible_after.iter().any(|candidate| candidate == after) {
            Ok(CommitmentResult::Confirmed)
        } else {
            Ok(CommitmentResult::Contradicted)
        }
    }

    /// Close the proof when the host cannot provide a trustworthy after-state.
    ///
    /// Indeterminate is terminal for this commitment. The caller must create
    /// a new commitment before retrying, preventing silent replay.
    pub fn resolve_indeterminate(
        &mut self,
        permit: &ExecutionPermit,
        scope: PredictionScope,
        command: u64,
        action: ActionId,
        before: &State,
    ) -> Result<CommitmentResult, ProofError> {
        self.check_binding(scope, command, action, before)?;
        self.check_permit(permit)?;
        if self.resolved {
            return Err(ProofError::CommitmentAlreadyResolved);
        }
        self.resolved = true;
        Ok(CommitmentResult::Indeterminate)
    }

    fn check_binding(
        &self,
        scope: PredictionScope,
        command: u64,
        action: ActionId,
        before: &State,
    ) -> Result<(), ProofError> {
        if self.scope != scope {
            return Err(ProofError::StaleScope);
        }
        if self.command != command {
            return Err(ProofError::WrongCommand);
        }
        if self.action != action {
            return Err(ProofError::WrongAction);
        }
        if self.before_digest != before.digest() {
            return Err(ProofError::BeforeStateMismatch);
        }
        Ok(())
    }

    fn check_permit(&self, permit: &ExecutionPermit) -> Result<(), ProofError> {
        if !permit.consumed {
            return Err(ProofError::PermitNotConsumed);
        }
        if permit.commitment_digest != self.commitment_digest {
            return Err(ProofError::StaleScope);
        }
        Ok(())
    }
}

/// Host authorization bound to one prediction commitment.
///
/// The host policy digest and nonce are carried through the trace so an
/// external executor can audit which policy authorized the action. The proof
/// boundary itself never executes an opaque action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionPermit {
    commitment_digest: Digest,
    scope: PredictionScope,
    command: u64,
    action: ActionId,
    before_digest: Digest,
    host_policy_digest: Digest,
    nonce: u64,
    consumed: bool,
}

impl ExecutionPermit {
    pub fn commitment_digest(&self) -> Digest { self.commitment_digest }
    pub fn scope(&self) -> PredictionScope { self.scope }
    pub fn command(&self) -> u64 { self.command }
    pub fn action(&self) -> ActionId { self.action }
    pub fn before_digest(&self) -> Digest { self.before_digest }
    pub fn host_policy_digest(&self) -> Digest { self.host_policy_digest }
    pub fn nonce(&self) -> u64 { self.nonce }
    pub fn consumed(&self) -> bool { self.consumed }
}

fn commitment_digest(
    scope: &PredictionScope,
    model_digest: Digest,
    command: u64,
    action: ActionId,
    before_digest: Digest,
    possible_after: &[State],
) -> Digest {
    let mut digest = mix(FNV_OFFSET, COMMITMENT_DOMAIN);
    digest = mix(digest, scope.schema_digest);
    digest = mix(digest, scope.abstraction_digest);
    match scope.context {
        Some(context) => {
            digest = mix(digest, 1);
            digest = mix(digest, context.0);
        }
        None => digest = mix(digest, 0),
    }
    digest = mix(digest, scope.epoch);
    digest = mix(digest, scope.episode);
    digest = mix(digest, scope.sequence);
    digest = mix(digest, model_digest);
    digest = mix(digest, command);
    digest = mix(digest, action);
    digest = mix(digest, before_digest);
    digest = mix(digest, possible_after.len() as u64);
    for outcome in possible_after {
        digest = mix(digest, outcome.digest());
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cross_domain_causal_core::{Literal, TransitionExample};

    fn learner() -> CausalLearner {
        let before = State::new([
            Literal::new(1, false),
            Literal::new(2, true),
            Literal::new(3, false),
        ]);
        let after = before.with(Literal::new(3, true));
        let mut learner = CausalLearner::new();
        learner
            .fit(&[TransitionExample::new(7, 9, before, after)])
            .expect("bounded positive witness should fit");
        learner
    }

    fn scope(sequence: u64) -> PredictionScope {
        PredictionScope {
            schema_digest: 10,
            abstraction_digest: 11,
            context: Some(ContextId(12)),
            epoch: 2,
            episode: 7,
            sequence,
        }
    }

    #[test]
    fn external_outcome_commitment_is_bounded_and_fail_closed() {
        let before = State::new([
            Literal::new(1, false),
            Literal::new(2, true),
        ]);
        assert_eq!(
            PredictionCommitment::from_bounded_outcomes(
                scope(0),
                100,
                9,
                42,
                &before,
                Vec::new(),
                1,
            ),
            Err(FrozenPredictionError::EmptyOutcomeSet)
        );
        assert_eq!(
            PredictionCommitment::from_bounded_outcomes(
                scope(0),
                100,
                9,
                42,
                &before,
                vec![before.clone(), before.with(Literal::new(2, false))],
                1,
            ),
            Err(FrozenPredictionError::OutcomeBudgetExceeded {
                observed: 2,
                limit: 1,
            })
        );
    }

    #[test]
    fn frozen_prediction_is_bounded_and_detached_from_source_revision() {
        let source = learner();
        let frozen = FrozenTransferModel::from_learner(&source, 4, 2).unwrap();
        let before = State::new([
            Literal::new(1, false),
            Literal::new(2, true),
            Literal::new(3, false),
        ]);
        let outcomes = frozen.predict(9, &before).unwrap();
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0], before.with(Literal::new(3, true)));
        assert!(frozen.is_unchanged(&source));
    }

    #[test]
    fn permit_requires_exact_scope_and_confirms_expected_outcome() {
        let source = learner();
        let frozen = FrozenTransferModel::from_learner(&source, 4, 2).unwrap();
        let before = State::new([
            Literal::new(1, false),
            Literal::new(2, true),
            Literal::new(3, false),
        ]);
        let expected = before.with(Literal::new(3, true));
        let mut commitment = frozen.commit(scope(1), 100, 9, &before).unwrap();
        let mut permit = commitment.issue_permit(500, 1);

        assert!(matches!(
            commitment.consume_permit(&mut permit, scope(2), 100, 9, &before),
            Err(ProofError::StaleScope)
        ));
        assert!(!permit.consumed());
        commitment
            .consume_permit(&mut permit, scope(1), 100, 9, &before)
            .unwrap();
        assert!(matches!(
            commitment.consume_permit(&mut permit, scope(1), 100, 9, &before),
            Err(ProofError::PermitAlreadyConsumed)
        ));
        assert_eq!(
            commitment
                .resolve(&permit, scope(1), 100, 9, &before, &expected)
                .unwrap(),
            CommitmentResult::Confirmed
        );
        assert!(commitment.is_resolved());
        assert_eq!(permit.host_policy_digest(), 500);
        assert_eq!(permit.nonce(), 1);
    }

    #[test]
    fn wrong_after_state_is_contradiction_and_unknown_action_abstains() {
        let source = learner();
        let frozen = FrozenTransferModel::from_learner(&source, 4, 2).unwrap();
        let before = State::new([
            Literal::new(1, false),
            Literal::new(2, true),
            Literal::new(3, false),
        ]);
        assert!(matches!(
            frozen.predict(10, &before),
            Err(FrozenPredictionError::UnknownAction)
        ));

        let wrong_after = before.with(Literal::new(2, false));
        let mut commitment = frozen.commit(scope(3), 100, 9, &before).unwrap();
        let mut permit = commitment.issue_permit(500, 2);
        commitment
            .consume_permit(&mut permit, scope(3), 100, 9, &before)
            .unwrap();
        assert_eq!(
            commitment
                .resolve(&permit, scope(3), 100, 9, &before, &wrong_after)
                .unwrap(),
            CommitmentResult::Contradicted
        );
    }

    #[test]
    fn indeterminate_is_terminal_and_cannot_be_replayed() {
        let source = learner();
        let frozen = FrozenTransferModel::from_learner(&source, 4, 2).unwrap();
        let before = State::new([
            Literal::new(1, false),
            Literal::new(2, true),
            Literal::new(3, false),
        ]);
        let mut commitment = frozen.commit(scope(4), 100, 9, &before).unwrap();
        let mut permit = commitment.issue_permit(500, 3);
        commitment
            .consume_permit(&mut permit, scope(4), 100, 9, &before)
            .unwrap();
        assert_eq!(
            commitment
                .resolve_indeterminate(&permit, scope(4), 100, 9, &before)
                .unwrap(),
            CommitmentResult::Indeterminate
        );
        assert!(matches!(
            commitment.resolve_indeterminate(&permit, scope(4), 100, 9, &before),
            Err(ProofError::CommitmentAlreadyResolved)
        ));
    }
}
