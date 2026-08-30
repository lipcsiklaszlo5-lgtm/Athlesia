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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelAssessment {
    concept: StructuralConcept,
    evidence: EvidenceState,
    status: RevisionStatus,
}

impl ModelAssessment {
    pub fn concept(&self) -> &StructuralConcept {
        &self.concept
    }

    pub const fn evidence(&self) -> EvidenceState {
        self.evidence
    }

    pub const fn status(&self) -> RevisionStatus {
        self.status
    }

    pub fn net_support(&self) -> i128 {
        i128::from(self.evidence.confirmations()) - i128::from(self.evidence.violations())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompetingModels {
    memory: RevisionMemory,
    policy: RevisionPolicy,
}

impl CompetingModels {
    pub fn new(policy: RevisionPolicy) -> Self {
        Self {
            memory: RevisionMemory::new(),
            policy,
        }
    }

    pub fn with_default_policy() -> Self {
        Self::new(RevisionPolicy::default())
    }

    pub fn len(&self) -> usize {
        self.memory.len()
    }

    pub fn is_empty(&self) -> bool {
        self.memory.is_empty()
    }

    pub fn record(
        &mut self,
        concept: StructuralConcept,
        observation: RevisionObservation,
    ) -> EvidenceState {
        self.memory.record(concept, observation)
    }

    pub fn assess(&self, concept: &StructuralConcept) -> Option<ModelAssessment> {
        let evidence = self.memory.evidence_for(concept)?;

        Some(ModelAssessment {
            concept: concept.clone(),
            evidence,
            status: self.policy.classify(evidence),
        })
    }

    pub fn assessments(&self) -> Vec<ModelAssessment> {
        let mut result: Vec<ModelAssessment> = self
            .memory
            .concepts()
            .map(|(concept, evidence)| ModelAssessment {
                concept: concept.clone(),
                evidence,
                status: self.policy.classify(evidence),
            })
            .collect();

        result.sort_by(|left, right| {
            status_rank(left.status)
                .cmp(&status_rank(right.status))
                .then_with(|| right.net_support().cmp(&left.net_support()))
                .then_with(|| right.evidence.total().cmp(&left.evidence.total()))
                .then_with(|| left.concept.cmp(&right.concept))
        });

        result
    }

    pub fn best(&self) -> Option<ModelAssessment> {
        self.assessments().into_iter().next()
    }
}

impl Default for CompetingModels {
    fn default() -> Self {
        Self::with_default_policy()
    }
}

const fn status_rank(status: RevisionStatus) -> u8 {
    match status {
        RevisionStatus::Supported => 0,
        RevisionStatus::Contested => 1,
        RevisionStatus::Unsupported => 2,
        RevisionStatus::Weakened => 3,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscriminativeExperiment {
    signature: athlesia::PrimitiveSignature,
    supporting_models: usize,
    opposing_models: usize,
}

impl DiscriminativeExperiment {
    pub const fn signature(&self) -> athlesia::PrimitiveSignature {
        self.signature
    }

    pub const fn supporting_models(&self) -> usize {
        self.supporting_models
    }

    pub const fn opposing_models(&self) -> usize {
        self.opposing_models
    }

    pub const fn discrimination_gain(&self) -> usize {
        self.supporting_models * self.opposing_models
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DiscriminativeExperimentSelector;

impl DiscriminativeExperimentSelector {
    pub const fn new() -> Self {
        Self
    }

    pub fn generate(&self, models: &CompetingModels) -> Vec<DiscriminativeExperiment> {
        use std::collections::BTreeSet;

        let assessments = models.assessments();

        let active: Vec<&ModelAssessment> = assessments
            .iter()
            .filter(|assessment| assessment.status() != RevisionStatus::Weakened)
            .collect();

        if active.len() < 2 {
            return Vec::new();
        }

        let signatures: BTreeSet<athlesia::PrimitiveSignature> = active
            .iter()
            .flat_map(|assessment| assessment.concept().signatures().iter().copied())
            .collect();

        let mut candidates: Vec<DiscriminativeExperiment> = signatures
            .into_iter()
            .filter_map(|signature| {
                let supporting_models = active
                    .iter()
                    .filter(|assessment| assessment.concept().contains(signature))
                    .count();

                let opposing_models = active.len() - supporting_models;

                if supporting_models == 0 || opposing_models == 0 {
                    return None;
                }

                Some(DiscriminativeExperiment {
                    signature,
                    supporting_models,
                    opposing_models,
                })
            })
            .collect();

        candidates.sort_by(|left, right| {
            right
                .discrimination_gain()
                .cmp(&left.discrimination_gain())
                .then_with(|| left.signature.cmp(&right.signature))
        });

        candidates
    }

    pub fn select(&self, models: &CompetingModels) -> Option<DiscriminativeExperiment> {
        self.generate(models).into_iter().next()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructuralObservationResult {
    Present,
    Absent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelEvidenceUpdate {
    concept: StructuralConcept,
    before: EvidenceState,
    after: EvidenceState,
    observation: StructuralObservationResult,
}

impl ModelEvidenceUpdate {
    pub fn concept(&self) -> &StructuralConcept {
        &self.concept
    }

    pub const fn before(&self) -> EvidenceState {
        self.before
    }

    pub const fn after(&self) -> EvidenceState {
        self.after
    }

    pub const fn observation(&self) -> StructuralObservationResult {
        self.observation
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EvidenceUpdateLoop;

impl EvidenceUpdateLoop {
    pub const fn new() -> Self {
        Self
    }

    pub fn apply(
        &self,
        models: &mut CompetingModels,
        experiment: &DiscriminativeExperiment,
        observation: StructuralObservationResult,
    ) -> Vec<ModelEvidenceUpdate> {
        let signature = experiment.signature();

        let mut targets: Vec<StructuralConcept> = models
            .assessments()
            .into_iter()
            .filter(|assessment| assessment.status() != RevisionStatus::Weakened)
            .filter(|assessment| assessment.concept().contains(signature))
            .map(|assessment| assessment.concept().clone())
            .collect();

        targets.sort();
        targets.dedup();

        let evidence_observation = match observation {
            StructuralObservationResult::Present => RevisionObservation::Confirmed,
            StructuralObservationResult::Absent => RevisionObservation::Violated,
        };

        targets
            .into_iter()
            .filter_map(|concept| {
                let before = models.assess(&concept)?.evidence();

                let after = models.record(concept.clone(), evidence_observation);

                Some(ModelEvidenceUpdate {
                    concept,
                    before,
                    after,
                    observation,
                })
            })
            .collect()
    }

    pub fn select_and_apply(
        &self,
        models: &mut CompetingModels,
        observation: StructuralObservationResult,
    ) -> Option<(DiscriminativeExperiment, Vec<ModelEvidenceUpdate>)> {
        let experiment = DiscriminativeExperimentSelector::new().select(models)?;

        let updates = self.apply(models, &experiment, observation);

        Some((experiment, updates))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevisionCycleTransition {
    experiment: DiscriminativeExperiment,
    observation: StructuralObservationResult,
    updates: Vec<ModelEvidenceUpdate>,
    ranking_before: Vec<ModelAssessment>,
    ranking_after: Vec<ModelAssessment>,
    best_before: Option<StructuralConcept>,
    best_after: Option<StructuralConcept>,
}

impl RevisionCycleTransition {
    pub fn experiment(&self) -> &DiscriminativeExperiment {
        &self.experiment
    }

    pub const fn observation(&self) -> StructuralObservationResult {
        self.observation
    }

    pub fn updates(&self) -> &[ModelEvidenceUpdate] {
        &self.updates
    }

    pub fn ranking_before(&self) -> &[ModelAssessment] {
        &self.ranking_before
    }

    pub fn ranking_after(&self) -> &[ModelAssessment] {
        &self.ranking_after
    }

    pub fn best_before(&self) -> Option<&StructuralConcept> {
        self.best_before.as_ref()
    }

    pub fn best_after(&self) -> Option<&StructuralConcept> {
        self.best_after.as_ref()
    }

    pub fn preference_changed(&self) -> bool {
        self.best_before != self.best_after
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RevisionCycleEngine;

impl RevisionCycleEngine {
    pub const fn new() -> Self {
        Self
    }

    pub fn step(
        &self,
        models: &mut CompetingModels,
        observation: StructuralObservationResult,
    ) -> Option<RevisionCycleTransition> {
        let ranking_before = models.assessments();

        let best_before = ranking_before
            .first()
            .map(|assessment| assessment.concept().clone());

        let experiment = DiscriminativeExperimentSelector::new().select(models)?;

        let updates = EvidenceUpdateLoop::new().apply(models, &experiment, observation);

        let ranking_after = models.assessments();

        let best_after = ranking_after
            .first()
            .map(|assessment| assessment.concept().clone());

        Some(RevisionCycleTransition {
            experiment,
            observation,
            updates,
            ranking_before,
            ranking_after,
            best_before,
            best_after,
        })
    }
}
