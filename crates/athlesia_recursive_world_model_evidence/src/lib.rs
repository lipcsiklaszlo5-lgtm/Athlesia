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
