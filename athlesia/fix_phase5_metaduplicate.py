#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

p = pathlib.Path("crates/athlesia-metalearner/src/lib.rs")
s = p.read_text()

# A hozzáfűzött duplikátum eltávolítása
duplicate_block = r'''

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

if duplicate_block not in s:
    print("[ERROR] A duplikátum blokk nem található.")
    sys.exit(1)

s = s.replace(duplicate_block, "")
write_file(p, s)
print("[1] Duplikátum eltávolítva.")

count = s.count("pub fn record_failure_in_context")
print(f"[INFO] record_failure_in_context előfordulások száma: {count}")
if count != 1:
    print("[ERROR] Még mindig nem pontosan egyszer van meg a metódus.")
    sys.exit(1)

# Metalearner tesztek futtatása
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

# Kernel tesztek futtatása
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

# Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Remove duplicate record_failure_in_context; complete Phase 5"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
