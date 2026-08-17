#!/usr/bin/env python3
import os, subprocess, sys, pathlib

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Knowledge Base lib.rs teljes újraírása a dokumentum szerint
write_file("crates/athlesia-knowledge/src/lib.rs", r'''
use athlesia_types::{Program, PrimName};

/// A DSL-könyvtár egy bejegyzése: egy makró, ami egy program.
#[derive(Debug, Clone)]
pub struct Macro {
    pub id: u64,
    pub name: String,
    pub program: Program,
    pub version_added: u64,
}

/// Fogalom (concept): makrók absztrakt, névvel ellátott csoportja.
#[derive(Debug, Clone)]
pub struct Concept {
    pub id: u64,
    pub name: String,
    pub macro_ids: Vec<u64>,
}

/// A tudásbázisban rögzített változások típusai.
#[derive(Debug, Clone)]
pub enum ChangeKind {
    AddPrimitive(PrimName),
    AddMacro { name: String },
    PruneMacro { name: String },
    AddConcept { name: String },
}

/// Egy audit bejegyzés.
#[derive(Debug, Clone)]
pub struct LibraryChange {
    pub version: u64,
    pub change: ChangeKind,
    pub evidence: Option<String>,
}

/// A Manhattan Kernel perzisztens tudástára.
/// Két nézet van: az aktív DSL-könyvtár (primitives, macros, concepts)
/// és az archívum (changes), ami minden módosítást verziózva tárol.
#[derive(Debug, Default)]
pub struct KnowledgeBase {
    pub primitives: Vec<PrimName>,
    pub macros: Vec<Macro>,
    pub concepts: Vec<Concept>,
    pub archive: Vec<LibraryChange>,
    pub version: u64,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_primitive(&mut self, prim: PrimName) {
        if !self.primitives.contains(&prim) {
            self.primitives.push(prim);
            self.version += 1;
            self.archive.push(LibraryChange {
                version: self.version,
                change: ChangeKind::AddPrimitive(prim),
                evidence: None,
            });
        }
    }

    pub fn add_macro(&mut self, name: String, program: Program) {
        let id = self.macros.len() as u64;
        self.macros.push(Macro {
            id,
            name: name.clone(),
            program,
            version_added: self.version + 1,
        });
        self.version += 1;
        self.archive.push(LibraryChange {
            version: self.version,
            change: ChangeKind::AddMacro { name },
            evidence: None,
        });
    }

    pub fn add_concept(&mut self, name: String, macro_ids: Vec<u64>) {
        let id = self.concepts.len() as u64;
        self.concepts.push(Concept {
            id,
            name: name.clone(),
            macro_ids,
        });
        self.version += 1;
        self.archive.push(LibraryChange {
            version: self.version,
            change: ChangeKind::AddConcept { name },
            evidence: None,
        });
    }

    pub fn get_macro_by_name(&self, name: &str) -> Option<&Macro> {
        self.macros.iter().find(|m| m.name == name)
    }

    pub fn get_concept_by_name(&self, name: &str) -> Option<&Concept> {
        self.concepts.iter().find(|c| c.name == name)
    }

    pub fn get_all_macros(&self) -> &[Macro] {
        &self.macros
    }
}
''')
print("[1] Knowledge Base lib.rs teljesen újraírva.")

# 2. Tesztek teljes újraírása a dokumentum szerint
write_file("crates/athlesia-knowledge/tests/knowledge_full_test.rs", r'''
use athlesia_knowledge::{KnowledgeBase, ChangeKind};
use athlesia_types::{PrimName, Params, Program};

#[test]
fn add_primitive_increases_version_and_archives() {
    let mut kb = KnowledgeBase::new();
    kb.add_primitive(PrimName::Translate);

    assert_eq!(kb.version, 1);
    assert_eq!(kb.primitives.len(), 1);
    assert_eq!(kb.archive.len(), 1);
    assert!(matches!(&kb.archive[0].change, ChangeKind::AddPrimitive(_)));
}

#[test]
fn add_macro_stores_program_and_increases_version() {
    let mut kb = KnowledgeBase::new();
    let program: Program = vec![(PrimName::ReflectH, Params::None)];

    kb.add_macro("mirror_h".to_string(), program.clone());

    assert_eq!(kb.version, 1);
    assert_eq!(kb.get_all_macros().len(), 1);
    assert!(kb.get_macro_by_name("mirror_h").is_some());
    assert_eq!(kb.archive.len(), 1);
}

#[test]
fn add_concept_stores_macro_refs_and_archives() {
    let mut kb = KnowledgeBase::new();
    kb.add_macro("macro1".to_string(), vec![(PrimName::Translate, Params::Translate(1,0))]);
    kb.add_macro("macro2".to_string(), vec![(PrimName::Rotate90, Params::None)]);

    // A két makró id-je: 0 és 1
    kb.add_concept("motion".to_string(), vec![0, 1]);

    assert_eq!(kb.version, 3); // 1 primitív? nem, csak macro-k: 1. macro -> version 1, 2. macro -> version 2, fogalom -> version 3
    assert_eq!(kb.concepts.len(), 1);
    assert!(kb.get_concept_by_name("motion").is_some());
    assert_eq!(kb.archive.len(), 3);
    assert!(matches!(&kb.archive[2].change, ChangeKind::AddConcept { name } if name == "motion"));
}

#[test]
fn do_not_duplicate_primitive() {
    let mut kb = KnowledgeBase::new();
    kb.add_primitive(PrimName::Translate);
    kb.add_primitive(PrimName::Translate);

    assert_eq!(kb.primitives.len(), 1);
    assert_eq!(kb.version, 1);
}
''')
print("[2] Knowledge Base tesztek hozzáadva.")

# 3. Tesztek futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-knowledge", "--test", "knowledge_full_test"], capture_output=True, text=True, check=False)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Knowledge Base tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Knowledge Base tesztek zöldek.")

# 4. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Finalize Knowledge Base module with concepts and evidence"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
