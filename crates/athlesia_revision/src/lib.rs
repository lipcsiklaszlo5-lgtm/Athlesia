use std::collections::BTreeMap;

use athlesia::{PredictionOutcome, StructuralConcept};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EvidenceState {
    confirmations: u64,
    violations: u64,
}

impl EvidenceState {
    pub const fn confirmations(self) -> u64 {
        self.confirmations
    }

    pub const fn violations(self) -> u64 {
        self.violations
    }

    pub const fn total(self) -> u64 {
        self.confirmations + self.violations
    }

    pub const fn is_contested(self) -> bool {
        self.confirmations > 0 && self.violations > 0
    }

    fn record(&mut self, observation: RevisionObservation) {
        match observation {
            RevisionObservation::Confirmed => {
                self.confirmations += 1;
            }
            RevisionObservation::Violated => {
                self.violations += 1;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RevisionObservation {
    Confirmed,
    Violated,
}

impl From<PredictionOutcome> for RevisionObservation {
    fn from(outcome: PredictionOutcome) -> Self {
        match outcome {
            PredictionOutcome::Confirmed => Self::Confirmed,
            PredictionOutcome::Violated => Self::Violated,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RevisionMemory {
    evidence: BTreeMap<StructuralConcept, EvidenceState>,
}

impl RevisionMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.evidence.len()
    }

    pub fn is_empty(&self) -> bool {
        self.evidence.is_empty()
    }

    pub fn record(
        &mut self,
        concept: StructuralConcept,
        observation: RevisionObservation,
    ) -> EvidenceState {
        let evidence = self.evidence.entry(concept).or_default();

        evidence.record(observation);

        *evidence
    }

    pub fn record_prediction_outcome(
        &mut self,
        concept: StructuralConcept,
        outcome: PredictionOutcome,
    ) -> EvidenceState {
        self.record(concept, RevisionObservation::from(outcome))
    }

    pub fn evidence_for(&self, concept: &StructuralConcept) -> Option<EvidenceState> {
        self.evidence.get(concept).copied()
    }

    pub fn concepts(&self) -> impl Iterator<Item = (&StructuralConcept, EvidenceState)> {
        self.evidence
            .iter()
            .map(|(concept, evidence)| (concept, *evidence))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RevisionStatus {
    Unsupported,
    Supported,
    Contested,
    Weakened,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RevisionPolicy {
    minimum_support: u64,
    weakening_margin: u64,
}

impl RevisionPolicy {
    pub const fn new(minimum_support: u64, weakening_margin: u64) -> Self {
        Self {
            minimum_support,
            weakening_margin,
        }
    }

    pub const fn minimum_support(self) -> u64 {
        self.minimum_support
    }

    pub const fn weakening_margin(self) -> u64 {
        self.weakening_margin
    }

    pub fn classify(self, evidence: EvidenceState) -> RevisionStatus {
        let confirmations = evidence.confirmations();
        let violations = evidence.violations();

        if confirmations == 0 && violations == 0 {
            return RevisionStatus::Unsupported;
        }

        if violations >= confirmations.saturating_add(self.weakening_margin) && violations > 0 {
            return RevisionStatus::Weakened;
        }

        if confirmations > 0 && violations > 0 {
            return RevisionStatus::Contested;
        }

        if confirmations >= self.minimum_support && violations == 0 {
            return RevisionStatus::Supported;
        }

        RevisionStatus::Unsupported
    }
}

impl Default for RevisionPolicy {
    fn default() -> Self {
        Self::new(2, 2)
    }
}
