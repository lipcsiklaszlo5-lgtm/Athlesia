
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


/// Igazolt fogalom: olyan fogalom, amelyet kísérletekkel megerősítettünk.
#[derive(Debug, Clone)]
pub struct VerifiedConcept {
    pub id: u64,
    pub name: String,
    pub relation_pattern: String,
    pub evidence_count: usize,
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
    pub verified_concepts: Vec<VerifiedConcept>,
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
}
