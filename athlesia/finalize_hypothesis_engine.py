#!/usr/bin/env python3
import os, subprocess, sys, pathlib

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Hypothesis Engine lib.rs teljes újraírása a dokumentum szerint
write_file("crates/athlesia-hypothesis/src/lib.rs", r'''
use athlesia_types::{Program, PrimName, Params, Color};
use athlesia_knowledge::KnowledgeBase;

/// A hipotézis-javasló interfész.
pub trait HypothesisProposer {
    fn propose(&self, kb: &KnowledgeBase) -> Vec<CandidateHypothesis>;
}

/// Jelölt hipotézis, amelynek a forrása és a programja is megvan.
#[derive(Debug, Clone)]
pub struct CandidateHypothesis {
    pub source: String,
    pub program: Program,
}

/// Statikus javasló: a tudásbázisban található primitívekből, makrókból
/// és fogalmakból generál jelölt programokat. Ez a legegyszerűbb,
/// determinisztikus választás. Az LLM-alapú javasló ezt fogja kiegészíteni.
pub struct StaticProposer;

impl HypothesisProposer for StaticProposer {
    fn propose(&self, kb: &KnowledgeBase) -> Vec<CandidateHypothesis> {
        let mut proposals = Vec::new();

        // Primitívek egyedi programokként
        for prim in &kb.primitives {
            let candidates = primitive_programs(prim);
            for program in candidates {
                proposals.push(CandidateHypothesis {
                    source: format!("primitive:{:?}", prim),
                    program,
                });
            }
        }

        // Makrók
        for m in &kb.macros {
            proposals.push(CandidateHypothesis {
                source: format!("macro:{}", m.name),
                program: m.program.clone(),
            });
        }

        // Fogalmak: a hozzájuk tartozó makrók programjai
        for concept in &kb.concepts {
            for macro_id in &concept.macro_ids {
                if let Some(m) = kb.macros.iter().find(|m| m.id == *macro_id) {
                    proposals.push(CandidateHypothesis {
                        source: format!("concept:{}:macro:{}", concept.name, m.name),
                        program: m.program.clone(),
                    });
                }
            }
        }

        proposals
    }
}

/// Egy primitívhez tartozó programváltozatok.
fn primitive_programs(prim: &PrimName) -> Vec<Program> {
    match prim {
        PrimName::Translate => {
            let mut v = Vec::new();
            for (dx, dy) in [(1,0), (0,1), (-1,0), (0,-1), (0,0)] {
                v.push(vec![(PrimName::Translate, Params::Translate(dx, dy))]);
            }
            v
        }
        PrimName::ReflectH => vec![vec![(PrimName::ReflectH, Params::None)]],
        PrimName::ReflectV => vec![vec![(PrimName::ReflectV, Params::None)]],
        PrimName::Rotate90 => vec![vec![(PrimName::Rotate90, Params::None)]],
        PrimName::Rotate180 => vec![vec![(PrimName::Rotate180, Params::None)]],
        PrimName::Rotate270 => vec![vec![(PrimName::Rotate270, Params::None)]],
        PrimName::Recolor => {
            let mut v = Vec::new();
            let perms: [[Color; 10]; 3] = [
                [Color(1), Color(0), Color(2), Color(3), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)],
                [Color(2), Color(1), Color(0), Color(3), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)],
                [Color(3), Color(2), Color(1), Color(0), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)],
            ];
            for perm in perms {
                v.push(vec![(PrimName::Recolor, Params::Recolor(perm))]);
            }
            v
        }
        PrimName::AddBorder => vec![vec![(PrimName::AddBorder, Params::None)]],
        PrimName::RemoveBorder => vec![vec![(PrimName::RemoveBorder, Params::None)]],
        PrimName::SwapColors => vec![
            vec![(PrimName::SwapColors, Params::SwapColors(1, 2))],
            vec![(PrimName::SwapColors, Params::SwapColors(1, 3))],
            vec![(PrimName::SwapColors, Params::SwapColors(2, 3))],
        ],
        PrimName::TranslateWrap => vec![
            vec![(PrimName::TranslateWrap, Params::TranslateWrap(1, 0))],
            vec![(PrimName::TranslateWrap, Params::TranslateWrap(0, 1))],
        ],
        PrimName::Tile => vec![vec![(PrimName::Tile, Params::None)]],
        PrimName::RepeatGrid => vec![vec![(PrimName::RepeatGrid, Params::None)]],
        _ => vec![],
    }
}
''')
print("[1] Hypothesis Engine lib.rs teljesen újraírva.")

# 2. Tesztek frissítése
write_file("crates/athlesia-hypothesis/tests/hypothesis_full_test.rs", r'''
use athlesia_hypothesis::{HypothesisProposer, StaticProposer, CandidateHypothesis};
use athlesia_knowledge::KnowledgeBase;
use athlesia_types::{PrimName, Params, Program};

#[test]
fn proposes_from_primitives() {
    let mut kb = KnowledgeBase::new();
    kb.add_primitive(PrimName::ReflectH);

    let proposer = StaticProposer;
    let proposals = proposer.propose(&kb);

    assert!(!proposals.is_empty());
    assert!(proposals.iter().any(|p| {
        matches!(&p.program.as_slice(), [(PrimName::ReflectH, Params::None)] )
    }));
}

#[test]
fn proposes_from_macros() {
    let mut kb = KnowledgeBase::new();
    let macro_program: Program = vec![(PrimName::Rotate90, Params::None)];
    kb.add_macro("rotate90".to_string(), macro_program.clone());

    let proposer = StaticProposer;
    let proposals = proposer.propose(&kb);

    assert!(proposals.iter().any(|p| p.program == macro_program));
    assert!(proposals.iter().any(|p| p.source.contains("macro:")));
}

#[test]
fn proposes_from_concepts() {
    let mut kb = KnowledgeBase::new();
    let macro1: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];
    let macro2: Program = vec![(PrimName::ReflectV, Params::None)];

    kb.add_macro("move_right".to_string(), macro1.clone());
    kb.add_macro("flip".to_string(), macro2.clone());
    kb.add_concept("motion".to_string(), vec![0, 1]);

    let proposer = StaticProposer;
    let proposals = proposer.propose(&kb);

    assert!(proposals.iter().any(|p| p.program == macro1));
    assert!(proposals.iter().any(|p| p.program == macro2));
    assert!(proposals.iter().any(|p| p.source.contains("concept:")));
}

#[test]
fn empty_knowledge_base_returns_no_proposals() {
    let kb = KnowledgeBase::new();
    let proposer = StaticProposer;
    let proposals = proposer.propose(&kb);
    assert!(proposals.is_empty());
}
''')
print("[2] Hypothesis Engine tesztek hozzáadva.")

# 3. Tesztek futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-hypothesis", "--test", "hypothesis_full_test"], capture_output=True, text=True, check=False)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Hypothesis Engine tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Hypothesis Engine tesztek zöldek.")

# 4. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Finalize Hypothesis Engine with candidate hypotheses and concept proposals"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
