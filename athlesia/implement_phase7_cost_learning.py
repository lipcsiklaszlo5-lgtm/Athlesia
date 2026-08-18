#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. MetaLearner bővítése: HypothesisScore új mezők, új metódusok
p = pathlib.Path("crates/athlesia-metalearner/src/lib.rs")
s = p.read_text()

# HypothesisScore struct bővítése
old_struct = '''#[derive(Debug, Clone, Copy, Default)]
pub struct HypothesisScore {
    pub successes: u32,
    pub failures: u32,
}'''
new_struct = '''#[derive(Debug, Clone, Copy, Default)]
pub struct HypothesisScore {
    pub successes: u32,
    pub failures: u32,
    pub total_cost: f64,
    pub cost_samples: u32,
}'''
if old_struct not in s:
    print("[ERROR] HypothesisScore struct nem található a várt formában.")
    sys.exit(1)
s = s.replace(old_struct, new_struct)

# Új impl blokk a fájl végéhez
new_impl = r'''

impl MetaLearner {
    /// Keresési költség rögzítése kontextusban.
    /// A költség (pl. megtett keresési lépések száma) átlagolva tárolódik.
    pub fn record_search_cost_in_context(&mut self, context: FeatureVector, hyp_id: u64, cost: f64) {
        let score = self
            .context_scores
            .entry((context, hyp_id))
            .or_insert(HypothesisScore::default());
        score.total_cost += cost;
        score.cost_samples += 1;

        if let Some(global) = self.global_scores.get_mut(&hyp_id) {
            global.total_cost += cost;
            global.cost_samples += 1;
        }
    }

    /// Átlagos keresési költség becslése adott kontextusban.
    /// Ha nincs elég adat, visszaesik a globális költségre,
    /// ha az sincs, akkor `None`-t ad.
    pub fn estimated_cost(&self, context: FeatureVector, hyp_id: u64) -> Option<f64> {
        if let Some(score) = self.context_scores.get(&(context, hyp_id)) {
            if score.cost_samples > 0 {
                return Some(score.total_cost / score.cost_samples as f64);
            }
        }
        if let Some(score) = self.global_scores.get(&hyp_id) {
            if score.cost_samples > 0 {
                return Some(score.total_cost / score.cost_samples as f64);
            }
        }
        None
    }
}
'''
s = s + new_impl
write_file(p, s)
print("[1] MetaLearner frissítve: költségtanulás hozzáadva.")

# 2. CognitiveController becslés frissítése
p = pathlib.Path("crates/athlesia-kernel/src/cognitive.rs")
s = p.read_text()

old_cost_line = "        let predicted_search_cost = 100.0 * (1.0 - conf);"
new_cost_line = '''        // A prediktált keresési költség becslése. Ha a meta learner már tanult
        // költséget, használjuk azt, különben konfidencia-alapú heurisztika.
        let predicted_search_cost = meta
            .estimated_cost(*features, 0)
            .unwrap_or_else(|| 100.0 * (1.0 - conf));'''

if old_cost_line not in s:
    print("[ERROR] predicted_search_cost sor nem található.")
    sys.exit(1)
s = s.replace(old_cost_line, new_cost_line)
write_file(p, s)
print("[2] cognitive.rs frissítve: estimated_cost használata.")

# 3. Teszt a költségtanuláshoz
test_code = r'''
use athlesia_metalearner::MetaLearner;
use athlesia_features::FeatureVector;

#[test]
fn record_and_estimate_search_cost() {
    let mut meta = MetaLearner::new();
    let context = FeatureVector::default();
    let hyp_id = 0;

    // Kezdetben nincs becslés
    assert!(meta.estimated_cost(context, hyp_id).is_none());

    // Költségek rögzítése
    meta.record_search_cost_in_context(context, hyp_id, 10.0);
    meta.record_search_cost_in_context(context, hyp_id, 20.0);

    let estimated = meta.estimated_cost(context, hyp_id).expect("Léteznie kell becslésnek");
    assert!((estimated - 15.0).abs() < 0.001, "Átlag 15.0 kell, de {} volt", estimated);
}
'''
write_file("crates/athlesia-metalearner/tests/cost_learning_test.rs", test_code)
print("[3] cost_learning_test.rs létrehozva.")

# 4. Metalearner tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-metalearner"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Metalearner tesztek nem mentek át.")
    sys.exit(1)

# 5. Kernel tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-kernel"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Kernel tesztek nem mentek át.")
    sys.exit(1)

print("\n[SUCCESS] Phase 7 tesztek zöldek.")

# 6. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 7: MetaLearner learns search cost in context; cognitive controller uses it"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
