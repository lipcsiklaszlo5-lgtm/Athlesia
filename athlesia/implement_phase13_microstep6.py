#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. KnowledgeBase bővítése VerifiedConcept típussal
p = pathlib.Path("crates/athlesia-knowledge/src/lib.rs")
s = p.read_text()

# Új típusok beszúrása a Concept struct után
anchor = '''/// Fogalom (concept): makrók absztrakt, névvel ellátott csoportja.
#[derive(Debug, Clone)]
pub struct Concept {
    pub id: u64,
    pub name: String,
    pub macro_ids: Vec<u64>,
}
'''

new_types = anchor + '''

/// Igazolt fogalom: olyan fogalom, amelyet kísérletekkel megerősítettünk.
#[derive(Debug, Clone)]
pub struct VerifiedConcept {
    pub id: u64,
    pub name: String,
    pub relation_pattern: String,
    pub evidence_count: usize,
}
'''

if anchor not in s:
    print("[ERROR] Concept blokk nem található.")
    sys.exit(1)
s = s.replace(anchor, new_types)

# KnowledgeBase struct bővítése
struct_anchor = '''#[derive(Debug, Default)]
pub struct KnowledgeBase {
    pub primitives: Vec<PrimName>,
    pub macros: Vec<Macro>,
    pub concepts: Vec<Concept>,
    pub archive: Vec<LibraryChange>,
    pub version: u64,
}
'''
struct_new = '''#[derive(Debug, Default)]
pub struct KnowledgeBase {
    pub primitives: Vec<PrimName>,
    pub macros: Vec<Macro>,
    pub concepts: Vec<Concept>,
    pub verified_concepts: Vec<VerifiedConcept>,
    pub archive: Vec<LibraryChange>,
    pub version: u64,
}
'''
if struct_anchor not in s:
    print("[ERROR] KnowledgeBase struct nem található.")
    sys.exit(1)
s = s.replace(struct_anchor, struct_new)

# Metódusok hozzáadása a KnowledgeBase impl-en belül
method_anchor = '''    pub fn get_all_macros(&self) -> &[Macro] {
        &self.macros
    }
'''
new_methods = method_anchor + '''

    /// Új igazolt fogalom hozzáadása.
    pub fn add_verified_concept(&mut self, name: String, relation_pattern: String, evidence_count: usize) {
        let id = self.verified_concepts.len() as u64;
        self.verified_concepts.push(VerifiedConcept {
            id,
            name,
            relation_pattern,
            evidence_count,
        });
        self.version += 1;
    }

    /// Az összes igazolt fogalom visszaadása.
    pub fn get_verified_concepts(&self) -> &[VerifiedConcept] {
        &self.verified_concepts
    }
'''
if method_anchor not in s:
    print("[ERROR] get_all_macros blokk nem található.")
    sys.exit(1)
s = s.replace(method_anchor, new_methods)

p.write_text(s)
print("[1] knowledge lib.rs frissítve: VerifiedConcept és KnowledgeBase metódusok.")

# 2. Új tesztfájl
test_code = r'''
use athlesia_knowledge::KnowledgeBase;

#[test]
fn add_and_retrieve_verified_concepts() {
    let mut kb = KnowledgeBase::new();
    kb.add_verified_concept("RepeatedInteraction".to_string(), "interaction(A,B)".to_string(), 3);

    let verified = kb.get_verified_concepts();
    assert_eq!(verified.len(), 1);
    assert_eq!(verified[0].name, "RepeatedInteraction");
    assert_eq!(verified[0].relation_pattern, "interaction(A,B)");
    assert_eq!(verified[0].evidence_count, 3);
}
'''
write_file("crates/athlesia-knowledge/tests/verified_concept_test.rs", test_code)
print("[2] verified_concept_test.rs létrehozva.")

# 3. Knowledge tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-knowledge"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Knowledge tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Knowledge tesztek zöldek.")

# 4. Teljes workspace teszt
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

# 5. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 6: add VerifiedConcept storage to KnowledgeBase"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
