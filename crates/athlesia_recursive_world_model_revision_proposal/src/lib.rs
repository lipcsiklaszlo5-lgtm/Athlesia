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
