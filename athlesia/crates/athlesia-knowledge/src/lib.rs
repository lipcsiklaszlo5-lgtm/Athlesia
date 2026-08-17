
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


    /// Makró eltávolítása név alapján.
    /// A makró archiválásra kerül, azaz bejegyezzük a változást,
    /// de magát a makrót eltávolítjuk az aktív könyvtárból.
    pub fn remove_macro(&mut self, name: &str) -> bool {
        if let Some(pos) = self.macros.iter().position(|m| m.name == name) {
            self.macros.remove(pos);
            self.version += 1;
            self.archive.push(LibraryChange {
                version: self.version,
                change: ChangeKind::PruneMacro { name: name.to_string() },
            });
            true
        } else {
            false
        }
    }

    /// Makró hátravitel (hideg tárolóba) – jelen implementációban ugyanaz,
    /// mint az eltávolítás, de a ChangeKind megkülönbözteti a későbbi visszaállításhoz.
    pub fn prune_macro(&mut self, name: &str) -> bool {
        self.remove_macro(name)
    }
    pub fn get_macro_by_name(&self, name: &str) -> Option<&Macro> {
        self.macros.iter().find(|m| m.name == name)
    }

    pub fn get_all_macros(&self) -> &[Macro] {
        &self.macros
    }
}
