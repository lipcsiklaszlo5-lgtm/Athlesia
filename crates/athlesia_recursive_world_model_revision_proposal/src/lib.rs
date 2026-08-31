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
