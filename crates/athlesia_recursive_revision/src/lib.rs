use std::collections::BTreeMap;

use athlesia_recursive::RecursiveConcept;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveEvidenceState {
    confirmations: usize,
    violations: usize,
}

impl RecursiveEvidenceState {
    pub const fn new() -> Self {
        Self {
            confirmations: 0,
            violations: 0,
        }
    }

    pub const fn confirmations(&self) -> usize {
        self.confirmations
    }

    pub const fn violations(&self) -> usize {
        self.violations
    }

    pub const fn observations(&self) -> usize {
        self.confirmations + self.violations
    }

    pub const fn balance(&self) -> isize {
        self.confirmations as isize - self.violations as isize
    }

    pub const fn is_unobserved(&self) -> bool {
        self.confirmations == 0 && self.violations == 0
    }

    pub fn confirm(&mut self) {
        self.confirmations += 1;
    }

    pub fn violate(&mut self) {
        self.violations += 1;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecursiveRevisionObservation {
    Confirmed,
    Violated,
}

impl RecursiveRevisionObservation {
    pub const fn from_confirmation(confirmed: bool) -> Self {
        if confirmed {
            Self::Confirmed
        } else {
            Self::Violated
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecursiveRevisionMemory {
    evidence: BTreeMap<RecursiveConcept, RecursiveEvidenceState>,
}

impl RecursiveRevisionMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.evidence.len()
    }

    pub fn is_empty(&self) -> bool {
        self.evidence.is_empty()
    }

    pub fn contains(&self, concept: &RecursiveConcept) -> bool {
        self.evidence.contains_key(concept)
    }

    pub fn evidence(&self, concept: &RecursiveConcept) -> Option<&RecursiveEvidenceState> {
        self.evidence.get(concept)
    }

    pub fn observe(
        &mut self,
        concept: RecursiveConcept,
        observation: RecursiveRevisionObservation,
    ) -> RecursiveEvidenceState {
        let state = self.evidence.entry(concept).or_default();

        match observation {
            RecursiveRevisionObservation::Confirmed => {
                state.confirm();
            }
            RecursiveRevisionObservation::Violated => {
                state.violate();
            }
        }

        *state
    }

    pub fn confirm(&mut self, concept: RecursiveConcept) -> RecursiveEvidenceState {
        self.observe(concept, RecursiveRevisionObservation::Confirmed)
    }

    pub fn violate(&mut self, concept: RecursiveConcept) -> RecursiveEvidenceState {
        self.observe(concept, RecursiveRevisionObservation::Violated)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&RecursiveConcept, &RecursiveEvidenceState)> {
        self.evidence.iter()
    }
}
