
use std::collections::HashMap;
use athlesia_types::{Program};
use athlesia_features::FeatureVector;

/// Egy hipotézis pontszáma.
#[derive(Debug, Clone, Copy, Default)]
pub struct HypothesisScore {
    pub successes: u32,
    pub failures: u32,
}

/// A Manhattan Kernel MetaLearner modulja.
///
/// - Globális és kontextusfüggő pontozás (Laplace-simított prioritás)
/// - Kudarc-archívum a ismert rossz mintázatokhoz
/// - Zárt alakú, determinisztikus absztrakció-minőség pontozó
#[derive(Debug, Default)]
pub struct MetaLearner {
    pub global_scores: HashMap<u64, HypothesisScore>,
    pub context_scores: HashMap<(FeatureVector, u64), HypothesisScore>,
    pub failure_archive: std::collections::HashSet<(FeatureVector, Program)>,
}

impl MetaLearner {
    pub fn new() -> Self {
        Self::default()
    }

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

    /// Globális prioritás (Laplace-simított).
    pub fn priority(&self, hyp_id: u64) -> f64 {
        match self.global_scores.get(&hyp_id) {
            Some(s) => Self::laplace_priority(s),
            None => 0.5,
        }
    }

    /// Kontextusfüggő prioritás. Ha nincs elég kontextus-minta, globálisra esik vissza.
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

    /// Globális rangsor prioritás szerint.
    pub fn rank(&self, ids: &[u64]) -> Vec<u64> {
        let mut sorted = ids.to_vec();
        sorted.sort_by(|a, b| {
            let pa = self.priority(*a);
            let pb = self.priority(*b);
            pb.partial_cmp(&pa).unwrap().then(a.cmp(b))
        });
        sorted
    }

    /// Kontextusfüggő rangsor.
    pub fn rank_in_context(&self, fv: FeatureVector, ids: &[u64]) -> Vec<u64> {
        let mut sorted = ids.to_vec();
        sorted.sort_by(|a, b| {
            let pa = self.priority_in_context(fv, *a);
            let pb = self.priority_in_context(fv, *b);
            pb.partial_cmp(&pa).unwrap().then(a.cmp(b))
        });
        sorted
    }

    /// Kudarc-mintázat rögzítése.
    pub fn record_failure_pattern(&mut self, fv: FeatureVector, program: Program) {
        self.failure_archive.insert((fv, program));
    }

    /// Ellenőrzi, hogy ez a mintázat ismert kudarc-e.
    pub fn is_known_failure(&self, fv: FeatureVector, program: &Program) -> bool {
        self.failure_archive.contains(&(fv, program.clone()))
    }


    /// Egyszerűségi pontszám (Occam-prior): a rövidebb program jobb.
    /// A komplexitás büntetése: minden lépésért 1.0, a nagyobb mélységért extra.
    pub fn simplicity_score(&self, program: &Program) -> f64 {
        let len = program.len() as f64;
        // Egyszerű lineáris büntetés: minél hosszabb, annál kisebb a pontszám.
        // 1.0 / (1.0 + len) -> 1.0 az üres programra, csökken a hosszal.
        1.0 / (1.0 + len)
    }

    /// Zárt alakú, determinisztikus lineáris absztrakció-minőség pontozó.
    ///
    /// A jellemzők:
    /// - méret: a program hossza
    /// - általánosíthatóság: 1.0, ha a program egynél többféle kontextusban sikeres
    /// - tömörítési nyereség: proxy, pl. méret * 2
    /// - strukturális mélység: a program hossza
    ///
    /// A súlyok előre rögzítettek, hogy determinisztikus maradjon.
    pub fn score_abstraction(&self, program: &Program) -> f64 {
        let size = program.len() as f64;
        let generality = 1.0; // jelenleg nem számoljuk, de a modellben szerepel
        let compression_gain = size * 2.0;
        let depth = size;

        // Súlyok: méret (0.5), általánosíthatóság (1.0), tömörítési nyereség (0.3), mélység (-0.2)
        let score = 0.5 * size + 1.0 * generality + 0.3 * compression_gain - 0.2 * depth;

        if score < 0.0 {
            0.0
        } else {
            score
        }
    }
}
