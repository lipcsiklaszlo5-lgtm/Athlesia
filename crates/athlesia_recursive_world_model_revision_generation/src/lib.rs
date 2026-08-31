use athlesia_recursive::RecursiveUnit;
use athlesia_recursive_world_model::RecursiveWorldRule;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionGenerationCandidate {
    target: RecursiveWorldRule,
    replacement: RecursiveWorldRule,
    basis: Vec<RecursiveUnit>,
}

impl RecursiveWorldRevisionGenerationCandidate {
    pub fn new(
        target: RecursiveWorldRule,
        replacement: RecursiveWorldRule,
        mut basis: Vec<RecursiveUnit>,
    ) -> Option<Self> {
        if target == replacement || basis.is_empty() {
            return None;
        }

        basis.sort();
        basis.dedup();

        Some(Self {
            target,
            replacement,
            basis,
        })
    }

    pub fn target(&self) -> &RecursiveWorldRule {
        &self.target
    }

    pub fn replacement(&self) -> &RecursiveWorldRule {
        &self.replacement
    }

    pub fn basis(&self) -> &[RecursiveUnit] {
        &self.basis
    }

    pub fn basis_count(&self) -> usize {
        self.basis.len()
    }

    pub fn contains_basis_unit(&self, unit: &RecursiveUnit) -> bool {
        self.basis.binary_search(unit).is_ok()
    }

    pub fn changes_premises(&self) -> bool {
        self.target.premises() != self.replacement.premises()
    }

    pub fn changes_conclusions(&self) -> bool {
        self.target.conclusions() != self.replacement.conclusions()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionGenerationCandidateSet {
    candidates: Vec<RecursiveWorldRevisionGenerationCandidate>,
}

impl RecursiveWorldRevisionGenerationCandidateSet {
    pub fn new(mut candidates: Vec<RecursiveWorldRevisionGenerationCandidate>) -> Self {
        candidates.sort();
        candidates.dedup();

        Self { candidates }
    }

    pub fn candidates(&self) -> &[RecursiveWorldRevisionGenerationCandidate] {
        &self.candidates
    }

    pub fn len(&self) -> usize {
        self.candidates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    pub fn contains(&self, candidate: &RecursiveWorldRevisionGenerationCandidate) -> bool {
        self.candidates.binary_search(candidate).is_ok()
    }

    pub fn candidates_for_target(
        &self,
        target: &RecursiveWorldRule,
    ) -> Vec<RecursiveWorldRevisionGenerationCandidate> {
        self.candidates
            .iter()
            .filter(|candidate| candidate.target() == target)
            .cloned()
            .collect()
    }

    pub fn candidates_for_replacement(
        &self,
        replacement: &RecursiveWorldRule,
    ) -> Vec<RecursiveWorldRevisionGenerationCandidate> {
        self.candidates
            .iter()
            .filter(|candidate| candidate.replacement() == replacement)
            .cloned()
            .collect()
    }
}

use athlesia_recursive_world_model_revision_proposal::{
    RecursiveWorldRevisionProposal, RecursiveWorldRevisionProposalSet,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionGenerationProposalBridge {
    candidates: RecursiveWorldRevisionGenerationCandidateSet,
    proposals: RecursiveWorldRevisionProposalSet,
}

impl RecursiveWorldRevisionGenerationProposalBridge {
    pub fn new(candidates: RecursiveWorldRevisionGenerationCandidateSet) -> Self {
        let proposals = RecursiveWorldRevisionProposalSet::new(
            candidates
                .candidates()
                .iter()
                .filter_map(|candidate| {
                    RecursiveWorldRevisionProposal::new(
                        candidate.target().clone(),
                        candidate.replacement().clone(),
                    )
                })
                .collect(),
        );

        Self {
            candidates,
            proposals,
        }
    }

    pub fn candidates(&self) -> &RecursiveWorldRevisionGenerationCandidateSet {
        &self.candidates
    }

    pub fn proposals(&self) -> &RecursiveWorldRevisionProposalSet {
        &self.proposals
    }

    pub fn proposal_count(&self) -> usize {
        self.proposals.len()
    }

    pub fn candidate_count(&self) -> usize {
        self.candidates.len()
    }

    pub fn candidates_for_proposal(
        &self,
        proposal: &RecursiveWorldRevisionProposal,
    ) -> Vec<RecursiveWorldRevisionGenerationCandidate> {
        self.candidates
            .candidates()
            .iter()
            .filter(|candidate| {
                candidate.target() == proposal.target()
                    && candidate.replacement() == proposal.replacement()
            })
            .cloned()
            .collect()
    }

    pub fn proposal_for_candidate(
        &self,
        candidate: &RecursiveWorldRevisionGenerationCandidate,
    ) -> Option<RecursiveWorldRevisionProposal> {
        RecursiveWorldRevisionProposal::new(
            candidate.target().clone(),
            candidate.replacement().clone(),
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionGenerationProposalBridgeBuilder;

impl RecursiveWorldRevisionGenerationProposalBridgeBuilder {
    pub fn build(
        candidates: RecursiveWorldRevisionGenerationCandidateSet,
    ) -> RecursiveWorldRevisionGenerationProposalBridge {
        RecursiveWorldRevisionGenerationProposalBridge::new(candidates)
    }
}

use athlesia_recursive_world_model::RecursiveWorldModel;

use athlesia_recursive_world_model_revision_proposal::{
    RecursiveWorldRejectedRevisionProposal, RecursiveWorldRevisionProposalValidationSet,
    RecursiveWorldRevisionProposalValidator, RecursiveWorldValidatedRevisionProposal,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionGenerationValidation {
    bridge: RecursiveWorldRevisionGenerationProposalBridge,
    validations: RecursiveWorldRevisionProposalValidationSet,
}

impl RecursiveWorldRevisionGenerationValidation {
    pub fn new(
        model: &RecursiveWorldModel,
        candidates: RecursiveWorldRevisionGenerationCandidateSet,
    ) -> Self {
        let bridge = RecursiveWorldRevisionGenerationProposalBridge::new(candidates);

        let validations =
            RecursiveWorldRevisionProposalValidator::validate_set(model, bridge.proposals());

        Self {
            bridge,
            validations,
        }
    }

    pub fn bridge(&self) -> &RecursiveWorldRevisionGenerationProposalBridge {
        &self.bridge
    }

    pub fn validations(&self) -> &RecursiveWorldRevisionProposalValidationSet {
        &self.validations
    }

    pub fn accepted(&self) -> &[RecursiveWorldValidatedRevisionProposal] {
        self.validations.accepted()
    }

    pub fn rejected(&self) -> &[RecursiveWorldRejectedRevisionProposal] {
        self.validations.rejected()
    }

    pub fn accepted_count(&self) -> usize {
        self.validations.accepted_count()
    }

    pub fn rejected_count(&self) -> usize {
        self.validations.rejected_count()
    }

    pub fn candidates_for_accepted(
        &self,
        accepted: &RecursiveWorldValidatedRevisionProposal,
    ) -> Vec<RecursiveWorldRevisionGenerationCandidate> {
        self.bridge.candidates_for_proposal(accepted.proposal())
    }

    pub fn candidates_for_rejected(
        &self,
        rejected: &RecursiveWorldRejectedRevisionProposal,
    ) -> Vec<RecursiveWorldRevisionGenerationCandidate> {
        self.bridge.candidates_for_proposal(rejected.proposal())
    }

    pub fn accepted_candidates(&self) -> Vec<RecursiveWorldRevisionGenerationCandidate> {
        let mut candidates = self
            .accepted()
            .iter()
            .flat_map(|accepted| self.candidates_for_accepted(accepted))
            .collect::<Vec<_>>();

        candidates.sort();
        candidates.dedup();

        candidates
    }

    pub fn rejected_candidates(&self) -> Vec<RecursiveWorldRevisionGenerationCandidate> {
        let mut candidates = self
            .rejected()
            .iter()
            .flat_map(|rejected| self.candidates_for_rejected(rejected))
            .collect::<Vec<_>>();

        candidates.sort();
        candidates.dedup();

        candidates
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionGenerationValidator;

impl RecursiveWorldRevisionGenerationValidator {
    pub fn validate(
        model: &RecursiveWorldModel,
        candidates: RecursiveWorldRevisionGenerationCandidateSet,
    ) -> RecursiveWorldRevisionGenerationValidation {
        RecursiveWorldRevisionGenerationValidation::new(model, candidates)
    }
}

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceAssessor, RecursiveWorldEvidenceRanking, RecursiveWorldEvidenceState,
};

use athlesia_recursive_world_model_revision_proposal::{
    RecursiveWorldRevisionProposalEvidenceScope, RecursiveWorldRevisionProposalEvidenceScoper,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionGenerationEvidenceScope {
    validation: RecursiveWorldRevisionGenerationValidation,
    evidence_ranking: RecursiveWorldEvidenceRanking,
    proposal_scope: RecursiveWorldRevisionProposalEvidenceScope,
    active_candidates: Vec<RecursiveWorldRevisionGenerationCandidate>,
    inactive_candidates: Vec<RecursiveWorldRevisionGenerationCandidate>,
    rejected_candidates: Vec<RecursiveWorldRevisionGenerationCandidate>,
}

impl RecursiveWorldRevisionGenerationEvidenceScope {
    pub fn new(
        model: &RecursiveWorldModel,
        evidence_state: &RecursiveWorldEvidenceState,
        candidates: RecursiveWorldRevisionGenerationCandidateSet,
    ) -> Self {
        let validation = RecursiveWorldRevisionGenerationValidation::new(model, candidates);

        let evidence_ranking = RecursiveWorldEvidenceRanking::new(
            RecursiveWorldEvidenceAssessor::assess_many(evidence_state, model.rules().to_vec()),
        );

        let proposal_scope = RecursiveWorldRevisionProposalEvidenceScoper::scope(
            &evidence_ranking,
            validation.validations(),
        );

        let mut active_candidates = proposal_scope
            .active()
            .iter()
            .flat_map(|accepted| validation.candidates_for_accepted(accepted))
            .collect::<Vec<_>>();

        let mut inactive_candidates = proposal_scope
            .inactive()
            .iter()
            .flat_map(|accepted| validation.candidates_for_accepted(accepted))
            .collect::<Vec<_>>();

        let mut rejected_candidates = validation.rejected_candidates();

        active_candidates.sort();
        active_candidates.dedup();

        inactive_candidates.sort();
        inactive_candidates.dedup();

        rejected_candidates.sort();
        rejected_candidates.dedup();

        Self {
            validation,
            evidence_ranking,
            proposal_scope,
            active_candidates,
            inactive_candidates,
            rejected_candidates,
        }
    }

    pub fn validation(&self) -> &RecursiveWorldRevisionGenerationValidation {
        &self.validation
    }

    pub fn evidence_ranking(&self) -> &RecursiveWorldEvidenceRanking {
        &self.evidence_ranking
    }

    pub fn proposal_scope(&self) -> &RecursiveWorldRevisionProposalEvidenceScope {
        &self.proposal_scope
    }

    pub fn pressured_rule(&self) -> Option<&RecursiveWorldRule> {
        self.proposal_scope.pressured_rule()
    }

    pub fn has_negative_pressure(&self) -> bool {
        self.proposal_scope.has_negative_pressure()
    }

    pub fn active_candidates(&self) -> &[RecursiveWorldRevisionGenerationCandidate] {
        &self.active_candidates
    }

    pub fn inactive_candidates(&self) -> &[RecursiveWorldRevisionGenerationCandidate] {
        &self.inactive_candidates
    }

    pub fn rejected_candidates(&self) -> &[RecursiveWorldRevisionGenerationCandidate] {
        &self.rejected_candidates
    }

    pub fn active_candidate_count(&self) -> usize {
        self.active_candidates.len()
    }

    pub fn inactive_candidate_count(&self) -> usize {
        self.inactive_candidates.len()
    }

    pub fn rejected_candidate_count(&self) -> usize {
        self.rejected_candidates.len()
    }

    pub fn accepted_candidate_count(&self) -> usize {
        self.active_candidates
            .len()
            .saturating_add(self.inactive_candidates.len())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionGenerationEvidenceScoper;

impl RecursiveWorldRevisionGenerationEvidenceScoper {
    pub fn scope(
        model: &RecursiveWorldModel,
        evidence_state: &RecursiveWorldEvidenceState,
        candidates: RecursiveWorldRevisionGenerationCandidateSet,
    ) -> RecursiveWorldRevisionGenerationEvidenceScope {
        RecursiveWorldRevisionGenerationEvidenceScope::new(model, evidence_state, candidates)
    }
}
