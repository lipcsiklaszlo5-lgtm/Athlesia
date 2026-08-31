use athlesia_recursive::RecursiveUnit;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRule {
    premises: Vec<RecursiveUnit>,
    conclusions: Vec<RecursiveUnit>,
}

impl RecursiveWorldRule {
    pub fn new(premises: Vec<RecursiveUnit>, conclusions: Vec<RecursiveUnit>) -> Option<Self> {
        if premises.is_empty() || conclusions.is_empty() {
            return None;
        }

        let mut premises = premises;

        premises.sort();
        premises.dedup();

        let mut conclusions = conclusions;

        conclusions.sort();
        conclusions.dedup();

        Some(Self {
            premises,
            conclusions,
        })
    }

    pub fn premises(&self) -> &[RecursiveUnit] {
        &self.premises
    }

    pub fn conclusions(&self) -> &[RecursiveUnit] {
        &self.conclusions
    }

    pub fn premise_count(&self) -> usize {
        self.premises.len()
    }

    pub fn conclusion_count(&self) -> usize {
        self.conclusions.len()
    }

    pub fn contains_premise(&self, unit: &RecursiveUnit) -> bool {
        self.premises.binary_search(unit).is_ok()
    }

    pub fn contains_conclusion(&self, unit: &RecursiveUnit) -> bool {
        self.conclusions.binary_search(unit).is_ok()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldModel {
    rules: Vec<RecursiveWorldRule>,
}

impl RecursiveWorldModel {
    pub fn new(mut rules: Vec<RecursiveWorldRule>) -> Self {
        rules.sort();
        rules.dedup();

        Self { rules }
    }

    pub fn rules(&self) -> &[RecursiveWorldRule] {
        &self.rules
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn contains(&self, rule: &RecursiveWorldRule) -> bool {
        self.rules.binary_search(rule).is_ok()
    }
}
