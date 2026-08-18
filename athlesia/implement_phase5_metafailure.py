#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. MetaLearner bővítése record_failure_in_context metódussal
p = pathlib.Path("crates/athlesia-metalearner/src/lib.rs")
s = p.read_text()

# Új impl blokk hozzáfűzése a fájl végéhez
new_impl = r'''

impl MetaLearner {
    /// Kudarc rögzítése kontextusban: növeli a failure számlálót.
    /// Ez csökkenti a hasonló kontextusú hipotézisek prioritását.
    pub fn record_failure_in_context(&mut self, context: FeatureVector, hyp_id: u64) {
        let score = self
            .context_scores
            .entry((context, hyp_id))
            .or_insert(HypothesisScore::default());
        score.failures += 1;

        if let Some(global) = self.global_scores.get_mut(&hyp_id) {
            global.failures += 1;
        }
    }
}
'''

s = s + new_impl
write_file(p, s)
print("[1] MetaLearner frissítve: record_failure_in_context hozzáadva.")

# 2. Kernel lib.rs módosítása: hívjuk meg a meta.record_failure_in_context-t
p = pathlib.Path("crates/athlesia-kernel/src/lib.rs")
s = p.read_text()

old_line = "            agent.wm.learn_from_error(id, &error);\n            agent.wm.record_prediction_error(error);"
new_line = "            agent.wm.learn_from_error(id, &error);\n            agent.wm.record_prediction_error(error);\n            agent.core.meta.record_failure_in_context(\n                athlesia_features::extract_features(&input_grid),\n                id,\n            );"

if old_line not in s:
    print("[ERROR] A kernel hibakezelő sora nem található.")
    sys.exit(1)
s = s.replace(old_line, new_line)
write_file(p, s)
print("[2] Kernel lib.rs frissítve: meta.record_failure_in_context hívása.")

# 3. Teszt a MetaLearner új metódusához
test_code = r'''
use athlesia_metalearner::MetaLearner;
use athlesia_features::FeatureVector;

#[test]
fn record_failure_decreases_priority() {
    let mut meta = MetaLearner::new();
    let context = FeatureVector::default();
    let hyp_id = 0;

    // Kezdetben semleges
    let initial = meta.priority_in_context(context, hyp_id);
    assert!((initial - 0.5).abs() < 0.001);

    // Néhány siker
    for _ in 0..3 {
        meta.record_success_in_context(context, hyp_id);
    }
    let after_success = meta.priority_in_context(context, hyp_id);
    assert!(after_success > 0.5);

    // Egy kudarc
    meta.record_failure_in_context(context, hyp_id);
    let after_failure = meta.priority_in_context(context, hyp_id);
    assert!(after_failure < after_success, "A kudarcnak csökkentenie kell a prioritást");
}
'''
write_file("crates/athlesia-metalearner/tests/failure_context_test.rs", test_code)
print("[3] failure_context_test.rs létrehozva.")

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

print("\n[SUCCESS] Phase 5 tesztek zöldek.")

# 6. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 5: MetaLearner records failures in context; kernel uses it on verification errors"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
