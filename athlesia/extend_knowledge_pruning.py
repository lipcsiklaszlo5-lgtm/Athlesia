#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
KB_DIR = os.path.join(PROJECT, "crates", "athlesia-knowledge")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. KnowledgeBase lib.rs bővítése pruning metódusokkal
lib_path = os.path.join(KB_DIR, "src", "lib.rs")
content = pathlib.Path(lib_path).read_text()

if "pub fn remove_macro" not in content:
    pruning_code = '''
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
'''
    # Beszúrás az impl KnowledgeBase blokkba, az add_macro után
    marker = "    pub fn get_macro_by_name"
    insertion_point = content.find(marker)
    if insertion_point == -1:
        print("[ERROR] Nem találom a get_macro_by_name markert.")
        sys.exit(1)
    content = content[:insertion_point] + pruning_code + content[insertion_point:]
    write_file(lib_path, content)
    print("[INFO] remove_macro és prune_macro hozzáadva.")
else:
    print("[INFO] remove_macro már létezik.")

# 2. Teszt hozzáadása
test_content = r'''
use athlesia_knowledge::{KnowledgeBase, ChangeKind};
use athlesia_types::{PrimName, Params, Program};

#[test]
fn remove_macro_deletes_and_archives() {
    let mut kb = KnowledgeBase::new();
    let program: Program = vec![(PrimName::ReflectH, Params::None)];
    kb.add_macro("mirror_h".to_string(), program.clone());

    assert_eq!(kb.get_all_macros().len(), 1);

    let removed = kb.remove_macro("mirror_h");
    assert!(removed);
    assert_eq!(kb.get_all_macros().len(), 0);
    assert!(kb.get_macro_by_name("mirror_h").is_none());

    // Archivumban a PruneMacro eseménynek kell lennie
    let last = kb.archive.last().expect("Kell lennie archív bejegyzésnek");
    match &last.change {
        ChangeKind::PruneMacro { name } => assert_eq!(name, "mirror_h"),
        _ => panic!("Hibás ChangeKind"),
    }
}

#[test]
fn remove_nonexistent_macro_returns_false() {
    let mut kb = KnowledgeBase::new();
    let removed = kb.remove_macro("nonexistent");
    assert!(!removed);
    assert_eq!(kb.version, 0);
}

#[test]
fn prune_macro_is_alias_for_remove() {
    let mut kb = KnowledgeBase::new();
    let program: Program = vec![(PrimName::Rotate90, Params::None)];
    kb.add_macro("rotate90".to_string(), program);

    assert!(kb.prune_macro("rotate90"));
    assert_eq!(kb.get_all_macros().len(), 0);
}
'''
write_file(os.path.join(KB_DIR, "tests", "pruning_test.rs"), test_content)
print("[INFO] Pruning tesztek hozzáadva.")

# 3. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-knowledge"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Knowledge pruning tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Knowledge pruning tesztek zöldek.")

# 4. Git commit és push a szülőből
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add pruning support to knowledge base"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
