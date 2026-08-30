use crate::ExperimentCandidate;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExperimentSelection {
    candidate: ExperimentCandidate,
}

impl ExperimentSelection {
    fn new(candidate: ExperimentCandidate) -> Self {
        Self { candidate }
    }

    pub const fn target(&self) -> usize {
        self.candidate.target()
    }

    pub fn information_gain(&self) -> usize {
        self.candidate.information_gain()
    }

    pub fn candidate(&self) -> &ExperimentCandidate {
        &self.candidate
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExperimentSelector;

impl ExperimentSelector {
    pub const fn new() -> Self {
        Self
    }

    pub fn select(&self, candidates: &[ExperimentCandidate]) -> Option<ExperimentSelection> {
        candidates
            .iter()
            .max_by(|left, right| {
                left.information_gain()
                    .cmp(&right.information_gain())
                    .then_with(|| right.target().cmp(&left.target()))
            })
            .cloned()
            .map(ExperimentSelection::new)
    }
}
