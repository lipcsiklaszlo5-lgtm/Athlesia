#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
ML_DIR = os.path.join(PROJECT, "crates", "athlesia-metalearner")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Cargo.toml kiegészítése az athlesia-features függőséggel
cargo_path = os.path.join(ML_DIR, "Cargo.toml")
cargo_content = pathlib.Path(cargo_path).read_text()
if "athlesia-features" not in cargo_content:
    cargo_content = cargo_content.replace(
        "[dependencies]\n",
        "[dependencies]\nathlesia-features = { path = \"../athlesia-features\" }\n"
    )
    write_file(cargo_path, cargo_content)
    print("[INFO] athlesia-features hozzáadva a metalearner függőségeihez.")

# 2. lib.rs cseréje kontextusfüggő pontozással
write_file(os.path.join(ML_DIR, "src", "lib.rs"), r'''
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
''')

print("[INFO] MetaLearner kontextusfüggő pontozás implementálva.")

# 3. Tesztek hozzáadása
write_file(os.path.join(ML_DIR, "tests", "context_metalearner_test.rs"), r'''
use athlesia_metalearner::MetaLearner;
use athlesia_features::FeatureVector;

fn fv(object_count: u8, touching_pairs: u8) -> FeatureVector {
    FeatureVector {
        object_count,
        color_counts: [0; 4],
        touching_pairs,
    }
}

#[test]
fn context_scores_change_ranking_only_when_enough_evidence() {
    let mut ml = MetaLearner::default();
    let ctx = fv(2, 1);
    let ids = vec![0, 1];

    // Globálisan a 0 legyen jobb
    ml.record_success(0);
    ml.record_success(0);
    ml.record_failure(1);

    // Kontextusban a 1 kapjon két sikert, a 0 kapjon két kudarcot
    for _ in 0..2 {
        ml.record_success_in_context(ctx, 1);
        ml.record_failure_in_context(ctx, 0);
    }

    let ranked = ml.rank_in_context(ctx, &ids);
    assert_eq!(ranked, vec![1, 0]);
}

#[test]
fn context_falls_back_to_global_when_insufficient_evidence() {
    let mut ml = MetaLearner::default();
    let ctx = fv(2, 1);
    let ids = vec![0, 1];

    ml.record_success(0);
    ml.record_failure(1);

    // Csak egy kontextus minta: nem elég, globális prioritás érvényesül
    ml.record_success_in_context(ctx, 0);

    let ranked = ml.rank_in_context(ctx, &ids);
    // globális: 0 jobb, mert 0>1
    assert_eq!(ranked, vec![0, 1]);
}
''')

print("[INFO] Kontextus tesztek hozzáadva.")

# 4. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-metalearner"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] MetaLearner kontextus tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] MetaLearner kontextus tesztek zöldek.")

# 5. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Extend MetaLearner with context-aware scoring"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
