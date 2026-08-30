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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveRevisionStatus {
    Unsupported,
    Weakened,
    Contested,
    Supported,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveRevisionPolicy;

impl RecursiveRevisionPolicy {
    pub const fn new() -> Self {
        Self
    }

    pub const fn classify(&self, evidence: &RecursiveEvidenceState) -> RecursiveRevisionStatus {
        match (evidence.confirmations() > 0, evidence.violations() > 0) {
            (false, false) => RecursiveRevisionStatus::Unsupported,
            (false, true) => RecursiveRevisionStatus::Weakened,
            (true, true) => RecursiveRevisionStatus::Contested,
            (true, false) => RecursiveRevisionStatus::Supported,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveModelAssessment {
    concept: RecursiveConcept,
    evidence: RecursiveEvidenceState,
    status: RecursiveRevisionStatus,
}

impl RecursiveModelAssessment {
    pub fn concept(&self) -> &RecursiveConcept {
        &self.concept
    }

    pub const fn evidence(&self) -> RecursiveEvidenceState {
        self.evidence
    }

    pub const fn status(&self) -> RecursiveRevisionStatus {
        self.status
    }

    pub const fn confirmations(&self) -> usize {
        self.evidence.confirmations()
    }

    pub const fn violations(&self) -> usize {
        self.evidence.violations()
    }

    pub const fn observations(&self) -> usize {
        self.evidence.observations()
    }

    pub const fn balance(&self) -> isize {
        self.evidence.balance()
    }
}

impl RecursiveRevisionMemory {
    pub fn assessment(&self, concept: &RecursiveConcept) -> RecursiveModelAssessment {
        let evidence = self.evidence(concept).copied().unwrap_or_default();

        let status = RecursiveRevisionPolicy::new().classify(&evidence);

        RecursiveModelAssessment {
            concept: concept.clone(),
            evidence,
            status,
        }
    }

    pub fn assessments(&self) -> Vec<RecursiveModelAssessment> {
        self.iter()
            .map(|(concept, evidence)| RecursiveModelAssessment {
                concept: concept.clone(),
                evidence: *evidence,
                status: RecursiveRevisionPolicy::new().classify(evidence),
            })
            .collect()
    }

    pub fn ranked_assessments(&self) -> Vec<RecursiveModelAssessment> {
        let mut assessments = self.assessments();

        assessments.sort_by(|left, right| {
            right
                .status()
                .cmp(&left.status())
                .then_with(|| right.balance().cmp(&left.balance()))
                .then_with(|| right.confirmations().cmp(&left.confirmations()))
                .then_with(|| left.violations().cmp(&right.violations()))
                .then_with(|| left.concept().cmp(right.concept()))
        });

        assessments
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecursiveCompetingModels {
    models: Vec<RecursiveModelAssessment>,
}

impl RecursiveCompetingModels {
    pub fn new(models: Vec<RecursiveModelAssessment>) -> Self {
        let mut models = models;

        models.sort_by(|left, right| {
            right
                .status()
                .cmp(&left.status())
                .then_with(|| right.balance().cmp(&left.balance()))
                .then_with(|| right.confirmations().cmp(&left.confirmations()))
                .then_with(|| left.violations().cmp(&right.violations()))
                .then_with(|| left.concept().cmp(right.concept()))
        });

        models.dedup_by(|left, right| left.concept() == right.concept());

        Self { models }
    }

    pub fn from_memory(memory: &RecursiveRevisionMemory) -> Self {
        Self::new(memory.ranked_assessments())
    }

    pub fn len(&self) -> usize {
        self.models.len()
    }

    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }

    pub fn models(&self) -> &[RecursiveModelAssessment] {
        &self.models
    }

    pub fn best(&self) -> Option<&RecursiveModelAssessment> {
        self.models.first()
    }

    pub fn runner_up(&self) -> Option<&RecursiveModelAssessment> {
        self.models.get(1)
    }

    pub fn supported_count(&self) -> usize {
        self.models
            .iter()
            .filter(|model| model.status() == RecursiveRevisionStatus::Supported)
            .count()
    }

    pub fn contested_count(&self) -> usize {
        self.models
            .iter()
            .filter(|model| model.status() == RecursiveRevisionStatus::Contested)
            .count()
    }

    pub fn weakened_count(&self) -> usize {
        self.models
            .iter()
            .filter(|model| model.status() == RecursiveRevisionStatus::Weakened)
            .count()
    }

    pub fn unsupported_count(&self) -> usize {
        self.models
            .iter()
            .filter(|model| model.status() == RecursiveRevisionStatus::Unsupported)
            .count()
    }
}
