
use std::collections::HashMap;
use athlesia_features::FeatureVector;

/// Hipotézis-pontszámok.
#[derive(Debug, Clone, Copy, Default)]
pub struct HypothesisScore {
    pub successes: u32,
    pub failures: u32,
}

/// MetaLearner: hipotézisek siker/kudarc alapú rangsorolása.
/// Két szintű pontozás:
/// - globális: minden hipotézis ID-hez tartozik egy Laplace-simított prioritás.
/// - kontextusfüggő: (FeatureVector, hipotézis ID) páronkénti pontok.
#[derive(Debug, Clone, Default)]
pub struct MetaLearner {
    pub global_scores: HashMap<u64, HypothesisScore>,
    pub context_scores: HashMap<(FeatureVector, u64), HypothesisScore>,
}

impl MetaLearner {
    pub fn record_success(&mut self, hyp_id: u64) {
        let entry = self.global_scores.entry(hyp_id).or_default();
        entry.successes += 1;
    }

    pub fn record_failure(&mut self, hyp_id: u64) {
        let entry = self.global_scores.entry(hyp_id).or_default();
        entry.failures += 1;
    }

    pub fn record_success_in_context(&mut self, fv: FeatureVector, hyp_id: u64) {
        let entry = self.context_scores.entry((fv, hyp_id)).or_default();
        entry.successes += 1;
        // Globális pontszám is nőjön
        self.record_success(hyp_id);
    }

    pub fn record_failure_in_context(&mut self, fv: FeatureVector, hyp_id: u64) {
        let entry = self.context_scores.entry((fv, hyp_id)).or_default();
        entry.failures += 1;
        self.record_failure(hyp_id);
    }

    fn laplace_priority(score: &HypothesisScore) -> f64 {
        (score.successes as f64 + 1.0) / (score.successes as f64 + score.failures as f64 + 2.0)
    }

    /// Globális prioritás.
    pub fn priority(&self, hyp_id: u64) -> f64 {
        match self.global_scores.get(&hyp_id) {
            Some(s) => Self::laplace_priority(s),
            None => 0.5,
        }
    }

    /// Kontextusfüggő prioritás, ha van elég minta; egyébként a globális.
    pub fn priority_in_context(&self, fv: FeatureVector, hyp_id: u64) -> f64 {
        match self.context_scores.get(&(fv, hyp_id)) {
            Some(s) => {
                let total = s.successes + s.failures;
                if total >= 2 {
                    Self::laplace_priority(s)
                } else {
                    self.priority(hyp_id)
                }
            }
            None => self.priority(hyp_id),
        }
    }

    /// Globális rangsorolás.
    pub fn rank(&self, ids: &[u64]) -> Vec<u64> {
        let mut sorted = ids.to_vec();
        sorted.sort_by(|a, b| {
            let pa = self.priority(*a);
            let pb = self.priority(*b);
            pb.partial_cmp(&pa).unwrap().then(a.cmp(b))
        });
        sorted
    }

    /// Kontextusfüggő rangsorolás: minden hipotézishez a kontextus-prioritást használja.
    pub fn rank_in_context(&self, fv: FeatureVector, ids: &[u64]) -> Vec<u64> {
        let mut sorted = ids.to_vec();
        sorted.sort_by(|a, b| {
            let pa = self.priority_in_context(fv, *a);
            let pb = self.priority_in_context(fv, *b);
            pb.partial_cmp(&pa).unwrap().then(a.cmp(b))
        });
        sorted
    }
}
