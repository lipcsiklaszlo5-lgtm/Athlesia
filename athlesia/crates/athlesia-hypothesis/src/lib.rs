
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


/// Absztrakt fogalomvázlat: relációs mintát ír le anélkül,
/// hogy konkrét primitívre vagy programra hivatkozna.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptSketch {
    pub name: String,
    pub relation_pattern: String,
    pub objects_involved: Vec<u64>,
}

/// Jelölt fogalom, amelyet még nem igazoltak.
#[derive(Debug, Clone, PartialEq)]
pub struct CandidateConcept {
    pub sketch: ConceptSketch,
    pub evidence: Vec<String>,
    pub confidence: f64,
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
