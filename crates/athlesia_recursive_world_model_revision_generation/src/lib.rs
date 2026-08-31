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
