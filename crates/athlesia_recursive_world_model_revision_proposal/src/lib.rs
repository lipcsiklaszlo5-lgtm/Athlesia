use athlesia_recursive_world_model::RecursiveWorldRule;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionProposal {
    target: RecursiveWorldRule,
    replacement: RecursiveWorldRule,
}

impl RecursiveWorldRevisionProposal {
    pub fn new(target: RecursiveWorldRule, replacement: RecursiveWorldRule) -> Option<Self> {
        if target == replacement {
            return None;
        }

        Some(Self {
            target,
            replacement,
        })
    }

    pub fn target(&self) -> &RecursiveWorldRule {
        &self.target
    }

    pub fn replacement(&self) -> &RecursiveWorldRule {
        &self.replacement
    }

    pub fn changes_premises(&self) -> bool {
        self.target.premises() != self.replacement.premises()
    }

    pub fn changes_conclusions(&self) -> bool {
        self.target.conclusions() != self.replacement.conclusions()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionProposalSet {
    proposals: Vec<RecursiveWorldRevisionProposal>,
}

impl RecursiveWorldRevisionProposalSet {
    pub fn new(mut proposals: Vec<RecursiveWorldRevisionProposal>) -> Self {
        proposals.sort();
        proposals.dedup();

        Self { proposals }
    }

    pub fn proposals(&self) -> &[RecursiveWorldRevisionProposal] {
        &self.proposals
    }

    pub fn len(&self) -> usize {
        self.proposals.len()
    }

    pub fn is_empty(&self) -> bool {
        self.proposals.is_empty()
    }

    pub fn contains(&self, proposal: &RecursiveWorldRevisionProposal) -> bool {
        self.proposals.binary_search(proposal).is_ok()
    }

    pub fn proposals_for_target(
        &self,
        target: &RecursiveWorldRule,
    ) -> Vec<RecursiveWorldRevisionProposal> {
        self.proposals
            .iter()
            .filter(|proposal| proposal.target() == target)
            .cloned()
            .collect()
    }
}

use athlesia_recursive_world_model::{RecursiveWorldMinimalRevision, RecursiveWorldModel};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldRevisionProposalRejection {
    TargetMissing,
    ReplacementCollision,
    RevisionUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldValidatedRevisionProposal {
    proposal: RecursiveWorldRevisionProposal,
    revision: RecursiveWorldMinimalRevision,
}

impl RecursiveWorldValidatedRevisionProposal {
    pub fn proposal(&self) -> &RecursiveWorldRevisionProposal {
        &self.proposal
    }

    pub fn target(&self) -> &RecursiveWorldRule {
        self.proposal.target()
    }

    pub fn replacement(&self) -> &RecursiveWorldRule {
        self.proposal.replacement()
    }

    pub fn revision(&self) -> &RecursiveWorldMinimalRevision {
        &self.revision
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecursiveWorldRevisionProposalValidation {
    Accepted(Box<RecursiveWorldValidatedRevisionProposal>),
    Rejected {
        proposal: RecursiveWorldRevisionProposal,
        reason: RecursiveWorldRevisionProposalRejection,
    },
}

impl RecursiveWorldRevisionProposalValidation {
    pub fn proposal(&self) -> &RecursiveWorldRevisionProposal {
        match self {
            Self::Accepted(validated) => validated.proposal(),

            Self::Rejected { proposal, .. } => proposal,
        }
    }

    pub fn validated(&self) -> Option<&RecursiveWorldValidatedRevisionProposal> {
        match self {
            Self::Accepted(validated) => Some(validated.as_ref()),

            Self::Rejected { .. } => None,
        }
    }

    pub const fn rejection_reason(&self) -> Option<RecursiveWorldRevisionProposalRejection> {
        match self {
            Self::Accepted(_) => None,

            Self::Rejected { reason, .. } => Some(*reason),
        }
    }

    pub fn is_accepted(&self) -> bool {
        matches!(self, Self::Accepted(_,))
    }

    pub fn is_rejected(&self) -> bool {
        !self.is_accepted()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionProposalValidator;

impl RecursiveWorldRevisionProposalValidator {
    pub fn validate(
        model: &RecursiveWorldModel,
        proposal: RecursiveWorldRevisionProposal,
    ) -> RecursiveWorldRevisionProposalValidation {
        if !model.contains(proposal.target()) {
            return RecursiveWorldRevisionProposalValidation::Rejected {
                proposal,
                reason: RecursiveWorldRevisionProposalRejection::TargetMissing,
            };
        }

        if model.contains(proposal.replacement()) {
            return RecursiveWorldRevisionProposalValidation::Rejected {
                proposal,
                reason: RecursiveWorldRevisionProposalRejection::ReplacementCollision,
            };
        }

        let revision = RecursiveWorldMinimalRevision::apply(
            model,
            proposal.target().clone(),
            proposal.replacement().clone(),
        );

        match revision {
            Some(revision) => RecursiveWorldRevisionProposalValidation::Accepted(Box::new(
                RecursiveWorldValidatedRevisionProposal { proposal, revision },
            )),

            None => RecursiveWorldRevisionProposalValidation::Rejected {
                proposal,
                reason: RecursiveWorldRevisionProposalRejection::RevisionUnavailable,
            },
        }
    }

    pub fn validate_many(
        model: &RecursiveWorldModel,
        proposals: &RecursiveWorldRevisionProposalSet,
    ) -> Vec<RecursiveWorldRevisionProposalValidation> {
        proposals
            .proposals()
            .iter()
            .cloned()
            .map(|proposal| Self::validate(model, proposal))
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRejectedRevisionProposal {
    proposal: RecursiveWorldRevisionProposal,
    reason: RecursiveWorldRevisionProposalRejection,
}

impl RecursiveWorldRejectedRevisionProposal {
    pub fn proposal(&self) -> &RecursiveWorldRevisionProposal {
        &self.proposal
    }

    pub const fn reason(&self) -> RecursiveWorldRevisionProposalRejection {
        self.reason
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionProposalValidationSet {
    accepted: Vec<RecursiveWorldValidatedRevisionProposal>,
    rejected: Vec<RecursiveWorldRejectedRevisionProposal>,
}

impl RecursiveWorldRevisionProposalValidationSet {
    pub fn new(validations: Vec<RecursiveWorldRevisionProposalValidation>) -> Self {
        let mut accepted = Vec::new();

        let mut rejected = Vec::new();

        for validation in validations {
            match validation {
                RecursiveWorldRevisionProposalValidation::Accepted(validated) => {
                    accepted.push(*validated);
                }

                RecursiveWorldRevisionProposalValidation::Rejected { proposal, reason } => {
                    rejected.push(RecursiveWorldRejectedRevisionProposal { proposal, reason });
                }
            }
        }

        accepted.sort_by(|left, right| left.proposal().cmp(right.proposal()));

        accepted.dedup_by(|left, right| left.proposal() == right.proposal());

        rejected.sort();
        rejected.dedup();

        Self { accepted, rejected }
    }

    pub fn accepted(&self) -> &[RecursiveWorldValidatedRevisionProposal] {
        &self.accepted
    }

    pub fn rejected(&self) -> &[RecursiveWorldRejectedRevisionProposal] {
        &self.rejected
    }

    pub fn accepted_count(&self) -> usize {
        self.accepted.len()
    }

    pub fn rejected_count(&self) -> usize {
        self.rejected.len()
    }

    pub fn len(&self) -> usize {
        self.accepted.len().saturating_add(self.rejected.len())
    }

    pub fn is_empty(&self) -> bool {
        self.accepted.is_empty() && self.rejected.is_empty()
    }

    pub fn revisions(&self) -> Vec<RecursiveWorldMinimalRevision> {
        self.accepted
            .iter()
            .map(|validated| validated.revision().clone())
            .collect()
    }

    pub fn accepted_for_target(
        &self,
        target: &RecursiveWorldRule,
    ) -> Vec<RecursiveWorldValidatedRevisionProposal> {
        self.accepted
            .iter()
            .filter(|validated| validated.target() == target)
            .cloned()
            .collect()
    }

    pub fn rejected_for_target(
        &self,
        target: &RecursiveWorldRule,
    ) -> Vec<RecursiveWorldRejectedRevisionProposal> {
        self.rejected
            .iter()
            .filter(|rejected| rejected.proposal().target() == target)
            .cloned()
            .collect()
    }
}

impl RecursiveWorldRevisionProposalValidator {
    pub fn validate_set(
        model: &RecursiveWorldModel,
        proposals: &RecursiveWorldRevisionProposalSet,
    ) -> RecursiveWorldRevisionProposalValidationSet {
        RecursiveWorldRevisionProposalValidationSet::new(Self::validate_many(model, proposals))
    }
}

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceAssessment, RecursiveWorldEvidenceRanking,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionProposalEvidenceScope {
    pressure: Option<RecursiveWorldEvidenceAssessment>,
    active: Vec<RecursiveWorldValidatedRevisionProposal>,
    inactive: Vec<RecursiveWorldValidatedRevisionProposal>,
    rejected: Vec<RecursiveWorldRejectedRevisionProposal>,
}

impl RecursiveWorldRevisionProposalEvidenceScope {
    pub fn new(
        ranking: &RecursiveWorldEvidenceRanking,
        validations: &RecursiveWorldRevisionProposalValidationSet,
    ) -> Self {
        let pressure = ranking
            .highest_revision_pressure()
            .filter(|assessment| assessment.balance() < 0)
            .cloned();

        let mut active = Vec::new();

        let mut inactive = Vec::new();

        for validated in validations.accepted() {
            if pressure
                .as_ref()
                .is_some_and(|assessment| validated.target() == assessment.rule())
            {
                active.push(validated.clone());
            } else {
                inactive.push(validated.clone());
            }
        }

        let mut rejected = validations.rejected().to_vec();

        active.sort_by(|left, right| left.proposal().cmp(right.proposal()));

        active.dedup_by(|left, right| left.proposal() == right.proposal());

        inactive.sort_by(|left, right| left.proposal().cmp(right.proposal()));

        inactive.dedup_by(|left, right| left.proposal() == right.proposal());

        rejected.sort();
        rejected.dedup();

        Self {
            pressure,
            active,
            inactive,
            rejected,
        }
    }

    pub fn pressure(&self) -> Option<&RecursiveWorldEvidenceAssessment> {
        self.pressure.as_ref()
    }

    pub fn pressured_rule(&self) -> Option<&RecursiveWorldRule> {
        self.pressure().map(|assessment| assessment.rule())
    }

    pub fn has_negative_pressure(&self) -> bool {
        self.pressure
            .as_ref()
            .is_some_and(|assessment| assessment.balance() < 0)
    }

    pub fn active(&self) -> &[RecursiveWorldValidatedRevisionProposal] {
        &self.active
    }

    pub fn inactive(&self) -> &[RecursiveWorldValidatedRevisionProposal] {
        &self.inactive
    }

    pub fn rejected(&self) -> &[RecursiveWorldRejectedRevisionProposal] {
        &self.rejected
    }

    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    pub fn inactive_count(&self) -> usize {
        self.inactive.len()
    }

    pub fn rejected_count(&self) -> usize {
        self.rejected.len()
    }

    pub fn accepted_count(&self) -> usize {
        self.active.len().saturating_add(self.inactive.len())
    }

    pub fn active_revisions(&self) -> Vec<RecursiveWorldMinimalRevision> {
        self.active
            .iter()
            .map(|validated| validated.revision().clone())
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionProposalEvidenceScoper;

impl RecursiveWorldRevisionProposalEvidenceScoper {
    pub fn scope(
        ranking: &RecursiveWorldEvidenceRanking,
        validations: &RecursiveWorldRevisionProposalValidationSet,
    ) -> RecursiveWorldRevisionProposalEvidenceScope {
        RecursiveWorldRevisionProposalEvidenceScope::new(ranking, validations)
    }
}

use athlesia_recursive_world_model::{
    RecursiveWorldRevisionActiveCycle, RecursiveWorldRevisionActiveCycleResult,
    RecursiveWorldRevisionBudget,
};

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceAssessor, RecursiveWorldEvidenceState,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionProposalCycleResult {
    validations: RecursiveWorldRevisionProposalValidationSet,
    evidence_ranking: RecursiveWorldEvidenceRanking,
    evidence_scope: RecursiveWorldRevisionProposalEvidenceScope,
    revision_cycle: RecursiveWorldRevisionActiveCycleResult,
}

impl RecursiveWorldRevisionProposalCycleResult {
    pub fn validations(&self) -> &RecursiveWorldRevisionProposalValidationSet {
        &self.validations
    }

    pub fn evidence_ranking(&self) -> &RecursiveWorldEvidenceRanking {
        &self.evidence_ranking
    }

    pub fn evidence_scope(&self) -> &RecursiveWorldRevisionProposalEvidenceScope {
        &self.evidence_scope
    }

    pub fn revision_cycle(&self) -> &RecursiveWorldRevisionActiveCycleResult {
        &self.revision_cycle
    }

    pub fn pressured_rule(&self) -> Option<&RecursiveWorldRule> {
        self.evidence_scope.pressured_rule()
    }

    pub fn active_proposal_count(&self) -> usize {
        self.evidence_scope.active_count()
    }

    pub fn rejected_proposal_count(&self) -> usize {
        self.validations.rejected_count()
    }

    pub fn selected_revision(&self) -> Option<&RecursiveWorldMinimalRevision> {
        self.revision_cycle
            .selected()
            .map(|costed| costed.revision())
    }

    pub fn revised_world(&self) -> Option<&RecursiveWorldModel> {
        self.revision_cycle.revised_world()
    }

    pub fn has_revision(&self) -> bool {
        self.revision_cycle.has_revision()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionProposalCycle;

impl RecursiveWorldRevisionProposalCycle {
    pub fn evaluate(
        model: &RecursiveWorldModel,
        evidence_state: &RecursiveWorldEvidenceState,
        proposals: &RecursiveWorldRevisionProposalSet,
        budget: RecursiveWorldRevisionBudget,
    ) -> RecursiveWorldRevisionProposalCycleResult {
        let validations = RecursiveWorldRevisionProposalValidator::validate_set(model, proposals);

        let assessments =
            RecursiveWorldEvidenceAssessor::assess_many(evidence_state, model.rules().to_vec());

        let evidence_ranking = RecursiveWorldEvidenceRanking::new(assessments);

        let evidence_scope =
            RecursiveWorldRevisionProposalEvidenceScope::new(&evidence_ranking, &validations);

        let revision_cycle =
            RecursiveWorldRevisionActiveCycle::evaluate(evidence_scope.active_revisions(), budget);

        RecursiveWorldRevisionProposalCycleResult {
            validations,
            evidence_ranking,
            evidence_scope,
            revision_cycle,
        }
    }
}
