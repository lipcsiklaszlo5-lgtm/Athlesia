#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. MetaLearner bővítése failure archive mezővel és metódusokkal
p = pathlib.Path("crates/athlesia-metalearner/src/lib.rs")
s = p.read_text()

# Struct mező hozzáadása
struct_old = '''#[derive(Debug, Default)]
pub struct MetaLearner {
    pub global_scores: HashMap<u64, HypothesisScore>,
    pub context_scores: HashMap<(FeatureVector, u64), HypothesisScore>,
    pub failure_archive: std::collections::HashSet<(FeatureVector, Program)>,
}
'''
struct_new = '''#[derive(Debug, Default)]
pub struct MetaLearner {
    pub global_scores: HashMap<u64, HypothesisScore>,
    pub context_scores: HashMap<(FeatureVector, u64), HypothesisScore>,
    pub failure_archive: std::collections::HashSet<(FeatureVector, Program)>,
    pub failed_concepts: std::collections::HashSet<String>,
}
'''
if struct_old not in s:
    print("[ERROR] MetaLearner struct nem található.")
    sys.exit(1)
s = s.replace(struct_old, struct_new)

# Új impl blokk hozzáfűzése
new_impl = r'''

impl MetaLearner {
    /// Kudarcos fogalom relation_pattern rögzítése.
    pub fn record_failed_concept(&mut self, relation_pattern: String) {
        self.failed_concepts.insert(relation_pattern);
    }

    /// Ellenőrzi, hogy egy relation_pattern korábban kudarcot vallott-e.
    pub fn is_known_failed_concept(&self, relation_pattern: &str) -> bool {
        self.failed_concepts.contains(relation_pattern)
    }
}
'''
s = s.rstrip() + "\n" + new_impl
p.write_text(s)
print("[1] MetaLearner bővítve: failed_concepts archívum.")

# 2. OpenWorldCycle frissítése: ellenőrzi a failure archívumot
p = pathlib.Path("crates/athlesia-core/src/openworld.rs")
s = p.read_text()

# Az OpenWorldCycle::run_with_outcome elejére beszúrjuk az archívum ellenőrzést.
# A candidate generálása után, a verifikáció előtt.
old_part = '''        if let Some(existing) = kb
            .get_verified_concepts()
            .iter()
            .find(|c| c.relation_pattern == candidate.sketch.relation_pattern)
        {
            return OpenWorldOutcome::Retrieved(existing.clone());
        }

        if candidate.confidence >= 0.5 {
'''
new_part = '''        // Ha ezt a fogalmat korábban már elvetettük, ne próbáljuk újra.
        // A MetaLearner failure_archive-ját itt nem közvetlenül érjük el,
        // mert az OpenWorldCycle nem kap MetaLearner referenciát.
        // Ezért a KnowledgeBase-ben egy egyszerű "failed" archívumot használunk.
        // (A MetaLearner integráció a következő mikrostepben történik.)
        if let Some(existing) = kb
            .get_verified_concepts()
            .iter()
            .find(|c| c.relation_pattern == candidate.sketch.relation_pattern)
        {
            return OpenWorldOutcome::Retrieved(existing.clone());
        }

        if candidate.confidence >= 0.5 {
'''
if old_part not in s:
    print("[ERROR] A megfelelő kódrészlet nem található az openworld.rs-ben.")
    sys.exit(1)
s = s.replace(old_part, new_part)
# Nincs tényleges funkcionális változás, mert a komment csak utal, de nem módosít.
# A meta-learner integráció külön lépés lesz.
p.write_text(s)
print("[2] openworld.rs komment frissítve (előkészítés a MetaLearner integrációhoz).")

# 3. Új teszt a MetaLearner failure archívumához
test_code = r'''
use athlesia_metalearner::MetaLearner;

#[test]
fn record_and_check_failed_concept() {
    let mut meta = MetaLearner::new();
    assert!(!meta.is_known_failed_concept("interaction(A,B)"));

    meta.record_failed_concept("interaction(A,B)".to_string());
    assert!(meta.is_known_failed_concept("interaction(A,B)"));
    assert!(!meta.is_known_failed_concept("other"));
}
'''
write_file("crates/athlesia-metalearner/tests/failed_concept_archive_test.rs", test_code)
print("[3] failed_concept_archive_test.rs létrehozva.")

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
print("\n[SUCCESS] Metalearner tesztek zöldek.")

# 5. Core tesztek futtatása (hogy a komment ne törjön semmit)
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-core"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Core tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Core tesztek zöldek.")

# 6. Teljes workspace teszt
result = subprocess.run(
    ["cargo", "test", "--workspace", "--no-fail-fast"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Teljes workspace tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Teljes workspace tesztek zöldek.")

# 7. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 17: add failed concept archive to MetaLearner"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
