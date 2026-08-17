
use std::collections::HashMap;

/// Hipotézis-pontszámok.
#[derive(Debug, Clone, Copy, Default)]
pub struct HypothesisScore {
    pub successes: u32,
    pub failures: u32,
}

/// MetaLearner: hipotézisek siker/kudarc alapú rangsorolása.
/// Jelenleg Laplace-simított prioritást használ:
/// (successes + 1) / (successes + failures + 2)
#[derive(Debug, Clone, Default)]
pub struct MetaLearner {
    pub scores: HashMap<u64, HypothesisScore>,
}

impl MetaLearner {
    pub fn record_success(&mut self, hyp_id: u64) {
        let entry = self.scores.entry(hyp_id).or_default();
        entry.successes += 1;
    }

    pub fn record_failure(&mut self, hyp_id: u64) {
        let entry = self.scores.entry(hyp_id).or_default();
        entry.failures += 1;
    }

    /// Prioritás: magasabb = jobb.
    pub fn priority(&self, hyp_id: u64) -> f64 {
        match self.scores.get(&hyp_id) {
            Some(s) => (s.successes as f64 + 1.0) / (s.successes as f64 + s.failures as f64 + 2.0),
            None => 0.5, // ismeretlen hipotézis semleges
        }
    }

    /// Hipotézis-azonosítókat sorba rendez prioritás szerint csökkenő sorrendben.
    /// Azonos prioritás esetén az id szerint növekvő sorrend (determinizmus).
    pub fn rank(&self, ids: &[u64]) -> Vec<u64> {
        let mut sorted = ids.to_vec();
        sorted.sort_by(|a, b| {
            let pa = self.priority(*a);
            let pb = self.priority(*b);
            pb.partial_cmp(&pa).unwrap().then(a.cmp(b))
        });
        sorted
    }
}
