#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
KB_DIR = os.path.join(PROJECT, "crates", "athlesia-knowledge")
WORKSPACE_TOML = os.path.join(PROJECT, "Cargo.toml")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Workspace frissítése
ws = pathlib.Path(WORKSPACE_TOML).read_text()
if "athlesia-knowledge" not in ws:
    ws = ws.replace(
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner", "crates/athlesia-verifier", "crates/athlesia-synthesis", "crates/athlesia-core", "crates/athlesia-search", "crates/athlesia-memory"]',
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner", "crates/athlesia-verifier", "crates/athlesia-synthesis", "crates/athlesia-core", "crates/athlesia-search", "crates/athlesia-memory", "crates/athlesia-knowledge"]'
    )
    write_file(WORKSPACE_TOML, ws)
    print("[INFO] Workspace frissítve.")

# 2. Knowledge Base crate létrehozása
os.makedirs(os.path.join(KB_DIR, "src"), exist_ok=True)
os.makedirs(os.path.join(KB_DIR, "tests"), exist_ok=True)

write_file(os.path.join(KB_DIR, "Cargo.toml"), '''[package]
name = "athlesia-knowledge"
version = "0.1.0"
edition = "2021"

[dependencies]
athlesia-types = { path = "../athlesia-types" }
''')

write_file(os.path.join(KB_DIR, "src", "lib.rs"), r'''
use athlesia_types::{Program, PrimName};

/// A DSL-könyvtár egy bejegyzése: egy makró, ami egy program.
#[derive(Debug, Clone)]
pub struct Macro {
    pub id: u64,
    pub name: String,
    pub program: Program,
    pub version_added: u64,
}

/// A tudásbázisban rögzített változások típusai.
#[derive(Debug, Clone)]
pub enum ChangeKind {
    AddPrimitive(PrimName),
    AddMacro { name: String },
    PruneMacro { name: String },
}

/// Egy audit bejegyzés.
#[derive(Debug, Clone)]
pub struct LibraryChange {
    pub version: u64,
    pub change: ChangeKind,
}

/// A Manhattan Kernel perzisztens tudástára.
/// Két nézet van: az aktív DSL-könyvtár (primitives, macros)
/// és az archívum (changes), ami minden módosítást verziózva tárol.
#[derive(Debug, Default)]
pub struct KnowledgeBase {
    pub primitives: Vec<PrimName>,
    pub macros: Vec<Macro>,
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
        });
    }

    pub fn get_macro_by_name(&self, name: &str) -> Option<&Macro> {
        self.macros.iter().find(|m| m.name == name)
    }

    pub fn get_all_macros(&self) -> &[Macro] {
        &self.macros
    }
}
''')

# 3. Tesztek
write_file(os.path.join(KB_DIR, "tests", "knowledge_test.rs"), r'''
use athlesia_knowledge::{KnowledgeBase, ChangeKind};
use athlesia_types::{PrimName, Params, Program};

#[test]
fn add_primitive_increases_version_and_archives() {
    let mut kb = KnowledgeBase::new();
    assert_eq!(kb.version, 0);

    kb.add_primitive(PrimName::Translate);
    assert_eq!(kb.version, 1);
    assert_eq!(kb.primitives.len(), 1);
    assert_eq!(kb.archive.len(), 1);
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
fn do_not_duplicate_primitive() {
    let mut kb = KnowledgeBase::new();
    kb.add_primitive(PrimName::Translate);
    kb.add_primitive(PrimName::Translate);

    assert_eq!(kb.primitives.len(), 1);
    assert_eq!(kb.version, 1);
}

#[test]
fn change_kind_is_recorded_correctly() {
    let mut kb = KnowledgeBase::new();
    kb.add_primitive(PrimName::Rotate90);

    if let Some(entry) = kb.archive.last() {
        match &entry.change {
            ChangeKind::AddPrimitive(p) => assert_eq!(*p, PrimName::Rotate90),
            _ => panic!("Hibás ChangeKind"),
        }
    } else {
        panic!("Nincs bejegyzés az archívumban");
    }
}
''')

print("[INFO] Knowledge Base crate létrehozva.")

# 4. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-knowledge"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Knowledge Base tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Knowledge Base tesztek zöldek.")

# 5. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add athlesia-knowledge module"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
