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
