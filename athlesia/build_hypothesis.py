#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
HYP_DIR = os.path.join(PROJECT, "crates", "athlesia-hypothesis")
WORKSPACE_TOML = os.path.join(PROJECT, "Cargo.toml")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Workspace frissítése
ws = pathlib.Path(WORKSPACE_TOML).read_text()
if "athlesia-hypothesis" not in ws:
    ws = ws.replace(
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner", "crates/athlesia-verifier", "crates/athlesia-synthesis", "crates/athlesia-core", "crates/athlesia-search", "crates/athlesia-memory", "crates/athlesia-knowledge", "crates/athlesia-abstraction"]',
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner", "crates/athlesia-verifier", "crates/athlesia-synthesis", "crates/athlesia-core", "crates/athlesia-search", "crates/athlesia-memory", "crates/athlesia-knowledge", "crates/athlesia-abstraction", "crates/athlesia-hypothesis"]'
    )
    write_file(WORKSPACE_TOML, ws)
    print("[INFO] Workspace frissítve.")

# 2. Hypothesis crate létrehozása
os.makedirs(os.path.join(HYP_DIR, "src"), exist_ok=True)
os.makedirs(os.path.join(HYP_DIR, "tests"), exist_ok=True)

write_file(os.path.join(HYP_DIR, "Cargo.toml"), '''[package]
name = "athlesia-hypothesis"
version = "0.1.0"
edition = "2021"

[dependencies]
athlesia-types = { path = "../athlesia-types" }
athlesia-knowledge = { path = "../athlesia-knowledge" }
''')

write_file(os.path.join(HYP_DIR, "src", "lib.rs"), r'''
use athlesia_types::{Program, PrimName, Params};
use athlesia_knowledge::KnowledgeBase;

/// A hipotézis-javasló interfész. A későbbiekben egy LLM-alapú implementáció
/// is elfoglalhatja ezt a helyet anélkül, hogy a rendszer más részei változnának.
pub trait HypothesisProposer {
    fn propose(&self, kb: &KnowledgeBase) -> Vec<Program>;
}

/// Statikus javasló: a tudásbázisban található primitívekből és makrókból
/// generál jelölt programokat. Ez a legegyszerűbb, determinisztikus választás.
/// Az LLM-alapú javasló ezt fogja kiegészíteni, nem pedig helyettesíteni.
pub struct StaticProposer;

impl HypothesisProposer for StaticProposer {
    fn propose(&self, kb: &KnowledgeBase) -> Vec<Program> {
        let mut proposals = Vec::new();

        // Primitívek egyedi programokként
        for prim in &kb.primitives {
            let program = match prim {
                PrimName::Translate => {
                    // Néhány alap eltolás
                    for (dx, dy) in [(1,0), (0,1), (0,0)] {
                        proposals.push(vec![(PrimName::Translate, Params::Translate(dx, dy))]);
                    }
                    continue;
                }
                PrimName::ReflectH => vec![(PrimName::ReflectH, Params::None)],
                PrimName::ReflectV => vec![(PrimName::ReflectV, Params::None)],
                PrimName::Rotate90 => vec![(PrimName::Rotate90, Params::None)],
                PrimName::Recolor => vec![(PrimName::Recolor, Params::Recolor([1,0,2,3]))],
            };
            proposals.push(program);
        }

        // Makrók programként
        for m in &kb.macros {
            proposals.push(m.program.clone());
        }

        proposals
    }
}
''')

# 3. Tesztek
write_file(os.path.join(HYP_DIR, "tests", "hypothesis_test.rs"), r'''
use athlesia_hypothesis::{HypothesisProposer, StaticProposer};
use athlesia_knowledge::KnowledgeBase;
use athlesia_types::{PrimName, Params, Program};

#[test]
fn proposes_macro_from_knowledge_base() {
    let mut kb = KnowledgeBase::new();
    let macro_program: Program = vec![(PrimName::ReflectH, Params::None)];
    kb.add_macro("mirror_h".to_string(), macro_program.clone());

    let proposer = StaticProposer;
    let proposals = proposer.propose(&kb);

    assert!(proposals.contains(&macro_program));
}

#[test]
fn proposes_primitives_from_knowledge_base() {
    let mut kb = KnowledgeBase::new();
    kb.add_primitive(PrimName::ReflectV);

    let proposer = StaticProposer;
    let proposals = proposer.propose(&kb);

    // A ReflectV primitív programnak meg kell jelennie
    let reflect_v: Program = vec![(PrimName::ReflectV, Params::None)];
    assert!(proposals.contains(&reflect_v));
}

#[test]
fn empty_knowledge_base_returns_no_proposals() {
    let kb = KnowledgeBase::new();
    let proposer = StaticProposer;
    let proposals = proposer.propose(&kb);
    assert!(proposals.is_empty());
}
''')

print("[INFO] Hypothesis crate létrehozva.")

# 4. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-hypothesis"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Hypothesis tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Hypothesis tesztek zöldek.")

# 5. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add athlesia-hypothesis module"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
