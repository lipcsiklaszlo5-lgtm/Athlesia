use athlesia_recursive::RecursiveUnit;
use athlesia_recursive_world_model::RecursiveWorldRule;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldEvidenceKind {
    Confirming,
    Violating,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldEvidenceRecord {
    rule: RecursiveWorldRule,
    observation: RecursiveUnit,
    kind: RecursiveWorldEvidenceKind,
}

impl RecursiveWorldEvidenceRecord {
    pub fn new(
        rule: RecursiveWorldRule,
        observation: RecursiveUnit,
        kind: RecursiveWorldEvidenceKind,
    ) -> Self {
        Self {
            rule,
            observation,
            kind,
        }
    }

    pub fn rule(&self) -> &RecursiveWorldRule {
        &self.rule
    }

    pub fn observation(&self) -> &RecursiveUnit {
        &self.observation
    }

    pub const fn kind(&self) -> RecursiveWorldEvidenceKind {
        self.kind
    }

    pub fn is_confirming(&self) -> bool {
        self.kind == RecursiveWorldEvidenceKind::Confirming
    }

    pub fn is_violating(&self) -> bool {
        self.kind == RecursiveWorldEvidenceKind::Violating
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldEvidenceSet {
    records: Vec<RecursiveWorldEvidenceRecord>,
}

impl RecursiveWorldEvidenceSet {
    pub fn new(mut records: Vec<RecursiveWorldEvidenceRecord>) -> Self {
        records.sort();
        records.dedup();

        Self { records }
    }

    pub fn records(&self) -> &[RecursiveWorldEvidenceRecord] {
        &self.records
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn contains(&self, record: &RecursiveWorldEvidenceRecord) -> bool {
        self.records.binary_search(record).is_ok()
    }

    pub fn confirming_count(&self) -> usize {
        self.records
            .iter()
            .filter(|record| record.is_confirming())
            .count()
    }

    pub fn violating_count(&self) -> usize {
        self.records
            .iter()
            .filter(|record| record.is_violating())
            .count()
    }

    pub fn records_for_rule(&self, rule: &RecursiveWorldRule) -> Vec<RecursiveWorldEvidenceRecord> {
        self.records
            .iter()
            .filter(|record| record.rule() == rule)
            .cloned()
            .collect()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldEvidenceState {
    evidence: RecursiveWorldEvidenceSet,
}

impl RecursiveWorldEvidenceState {
    pub fn new(evidence: RecursiveWorldEvidenceSet) -> Self {
        Self { evidence }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn evidence(&self) -> &RecursiveWorldEvidenceSet {
        &self.evidence
    }

    pub fn len(&self) -> usize {
        self.evidence.len()
    }

    pub fn is_empty(&self) -> bool {
        self.evidence.is_empty()
    }

    pub fn contains(&self, record: &RecursiveWorldEvidenceRecord) -> bool {
        self.evidence.contains(record)
    }

    pub fn accumulate(&self, record: RecursiveWorldEvidenceRecord) -> Self {
        let mut records = self.evidence.records().to_vec();

        records.push(record);

        Self {
            evidence: RecursiveWorldEvidenceSet::new(records),
        }
    }

    pub fn accumulate_many(&self, records: Vec<RecursiveWorldEvidenceRecord>) -> Self {
        let mut combined = self.evidence.records().to_vec();

        combined.extend(records);

        Self {
            evidence: RecursiveWorldEvidenceSet::new(combined),
        }
    }

    pub fn confirming_count(&self) -> usize {
        self.evidence.confirming_count()
    }

    pub fn violating_count(&self) -> usize {
        self.evidence.violating_count()
    }

    pub fn records_for_rule(&self, rule: &RecursiveWorldRule) -> Vec<RecursiveWorldEvidenceRecord> {
        self.evidence.records_for_rule(rule)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldEvidenceAccumulator;

impl RecursiveWorldEvidenceAccumulator {
    pub fn accumulate(
        state: &RecursiveWorldEvidenceState,
        record: RecursiveWorldEvidenceRecord,
    ) -> RecursiveWorldEvidenceState {
        state.accumulate(record)
    }

    pub fn accumulate_many(
        state: &RecursiveWorldEvidenceState,
        records: Vec<RecursiveWorldEvidenceRecord>,
    ) -> RecursiveWorldEvidenceState {
        state.accumulate_many(records)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveWorldEvidenceProfile {
    None,
    ViolatingOnly,
    Mixed,
    ConfirmingOnly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldEvidenceAssessment {
    rule: RecursiveWorldRule,
    confirming_count: usize,
    violating_count: usize,
    balance: i128,
    profile: RecursiveWorldEvidenceProfile,
}

impl RecursiveWorldEvidenceAssessment {
    pub fn evaluate(state: &RecursiveWorldEvidenceState, rule: RecursiveWorldRule) -> Self {
        let records = state.records_for_rule(&rule);

        let confirming_count = records
            .iter()
            .filter(|record| record.is_confirming())
            .count();

        let violating_count = records
            .iter()
            .filter(|record| record.is_violating())
            .count();

        let balance = confirming_count as i128 - violating_count as i128;

        let profile = match (confirming_count > 0, violating_count > 0) {
            (false, false) => RecursiveWorldEvidenceProfile::None,

            (true, false) => RecursiveWorldEvidenceProfile::ConfirmingOnly,

            (false, true) => RecursiveWorldEvidenceProfile::ViolatingOnly,

            (true, true) => RecursiveWorldEvidenceProfile::Mixed,
        };

        Self {
            rule,
            confirming_count,
            violating_count,
            balance,
            profile,
        }
    }

    pub fn rule(&self) -> &RecursiveWorldRule {
        &self.rule
    }

    pub const fn confirming_count(&self) -> usize {
        self.confirming_count
    }

    pub const fn violating_count(&self) -> usize {
        self.violating_count
    }

    pub const fn evidence_count(&self) -> usize {
        self.confirming_count.saturating_add(self.violating_count)
    }

    pub const fn balance(&self) -> i128 {
        self.balance
    }

    pub const fn profile(&self) -> RecursiveWorldEvidenceProfile {
        self.profile
    }

    pub fn has_evidence(&self) -> bool {
        self.evidence_count() > 0
    }

    pub fn is_mixed(&self) -> bool {
        self.profile == RecursiveWorldEvidenceProfile::Mixed
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldEvidenceAssessor;

impl RecursiveWorldEvidenceAssessor {
    pub fn assess(
        state: &RecursiveWorldEvidenceState,
        rule: RecursiveWorldRule,
    ) -> RecursiveWorldEvidenceAssessment {
        RecursiveWorldEvidenceAssessment::evaluate(state, rule)
    }

    pub fn assess_many(
        state: &RecursiveWorldEvidenceState,
        rules: Vec<RecursiveWorldRule>,
    ) -> Vec<RecursiveWorldEvidenceAssessment> {
        let mut rules = rules;

        rules.sort();
        rules.dedup();

        rules
            .into_iter()
            .map(|rule| Self::assess(state, rule))
            .collect()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldEvidenceRanking {
    assessments: Vec<RecursiveWorldEvidenceAssessment>,
}

impl RecursiveWorldEvidenceRanking {
    pub fn new(mut assessments: Vec<RecursiveWorldEvidenceAssessment>) -> Self {
        assessments.sort_by(|left, right| {
            right
                .violating_count()
                .cmp(&left.violating_count())
                .then_with(|| left.balance().cmp(&right.balance()))
                .then_with(|| right.evidence_count().cmp(&left.evidence_count()))
                .then_with(|| left.rule().cmp(right.rule()))
        });

        assessments.dedup();

        Self { assessments }
    }

    pub fn assessments(&self) -> &[RecursiveWorldEvidenceAssessment] {
        &self.assessments
    }

    pub fn len(&self) -> usize {
        self.assessments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.assessments.is_empty()
    }

    pub fn highest_revision_pressure(&self) -> Option<&RecursiveWorldEvidenceAssessment> {
        self.assessments.first()
    }
}

use athlesia_recursive_world_model::RecursiveWorldMinimalRevision;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldEvidenceRevisionBridge {
    pressure: Option<RecursiveWorldEvidenceAssessment>,
    candidates: Vec<RecursiveWorldMinimalRevision>,
}

impl RecursiveWorldEvidenceRevisionBridge {
    pub fn new(
        ranking: &RecursiveWorldEvidenceRanking,
        revisions: Vec<RecursiveWorldMinimalRevision>,
    ) -> Self {
        let pressure = ranking
            .highest_revision_pressure()
            .filter(|assessment| assessment.balance() < 0)
            .cloned();

        let mut candidates = match pressure.as_ref() {
            Some(assessment) => revisions
                .into_iter()
                .filter(|revision| revision.target() == assessment.rule())
                .collect(),

            None => Vec::new(),
        };

        candidates.sort_by(|left, right| {
            left.target()
                .cmp(right.target())
                .then_with(|| left.replacement().cmp(right.replacement()))
        });

        candidates.dedup();

        Self {
            pressure,
            candidates,
        }
    }

    pub fn pressure(&self) -> Option<&RecursiveWorldEvidenceAssessment> {
        self.pressure.as_ref()
    }

    pub fn pressured_rule(&self) -> Option<&RecursiveWorldRule> {
        self.pressure().map(|assessment| assessment.rule())
    }

    pub fn candidates(&self) -> &[RecursiveWorldMinimalRevision] {
        &self.candidates
    }

    pub fn len(&self) -> usize {
        self.candidates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    pub fn has_negative_pressure(&self) -> bool {
        self.pressure
            .as_ref()
            .is_some_and(|assessment| assessment.balance() < 0)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldEvidenceRevisionBridgeBuilder;

impl RecursiveWorldEvidenceRevisionBridgeBuilder {
    pub fn build(
        ranking: &RecursiveWorldEvidenceRanking,
        revisions: Vec<RecursiveWorldMinimalRevision>,
    ) -> RecursiveWorldEvidenceRevisionBridge {
        RecursiveWorldEvidenceRevisionBridge::new(ranking, revisions)
    }
}

use athlesia_recursive_world_model::{
    RecursiveWorldRevisionActiveCycle, RecursiveWorldRevisionActiveCycleResult,
    RecursiveWorldRevisionBudget,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldEvidenceRevisionCycleResult {
    ranking: RecursiveWorldEvidenceRanking,
    bridge: RecursiveWorldEvidenceRevisionBridge,
    revision_cycle: RecursiveWorldRevisionActiveCycleResult,
}

impl RecursiveWorldEvidenceRevisionCycleResult {
    pub fn ranking(&self) -> &RecursiveWorldEvidenceRanking {
        &self.ranking
    }

    pub fn bridge(&self) -> &RecursiveWorldEvidenceRevisionBridge {
        &self.bridge
    }

    pub fn revision_cycle(&self) -> &RecursiveWorldRevisionActiveCycleResult {
        &self.revision_cycle
    }

    pub fn pressured_rule(&self) -> Option<&RecursiveWorldRule> {
        self.bridge.pressured_rule()
    }

    pub fn selected_revision(&self) -> Option<&RecursiveWorldMinimalRevision> {
        self.revision_cycle
            .selected()
            .map(|costed| costed.revision())
    }

    pub fn revised_world(&self) -> Option<&athlesia_recursive_world_model::RecursiveWorldModel> {
        self.revision_cycle.revised_world()
    }

    pub fn has_revision(&self) -> bool {
        self.revision_cycle.has_revision()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldEvidenceRevisionCycle;

impl RecursiveWorldEvidenceRevisionCycle {
    pub fn evaluate(
        state: &RecursiveWorldEvidenceState,
        rules: Vec<RecursiveWorldRule>,
        revisions: Vec<RecursiveWorldMinimalRevision>,
        budget: RecursiveWorldRevisionBudget,
    ) -> RecursiveWorldEvidenceRevisionCycleResult {
        let assessments = RecursiveWorldEvidenceAssessor::assess_many(state, rules);

        let ranking = RecursiveWorldEvidenceRanking::new(assessments);

        let bridge = RecursiveWorldEvidenceRevisionBridge::new(&ranking, revisions);

        let revision_cycle =
            RecursiveWorldRevisionActiveCycle::evaluate(bridge.candidates().to_vec(), budget);

        RecursiveWorldEvidenceRevisionCycleResult {
            ranking,
            bridge,
            revision_cycle,
        }
    }
}
