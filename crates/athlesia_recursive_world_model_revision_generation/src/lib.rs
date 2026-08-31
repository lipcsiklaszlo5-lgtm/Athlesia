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
