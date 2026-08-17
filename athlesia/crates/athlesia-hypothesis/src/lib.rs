
use athlesia_types::{Program, PrimName, Params, Color};
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
                PrimName::Rotate180 => vec![(PrimName::Rotate180, Params::None)],
                PrimName::Rotate270 => vec![(PrimName::Rotate270, Params::None)],
                PrimName::Recolor => vec![(PrimName::Recolor, Params::Recolor([Color(1), Color(0), Color(2), Color(3)]))],
                PrimName::AddBorder => vec![(PrimName::AddBorder, Params::None)],
                PrimName::RemoveBorder => vec![(PrimName::RemoveBorder, Params::None)],
                PrimName::SwapColors => vec![(PrimName::SwapColors, Params::SwapColors(1, 2))],
                PrimName::TranslateWrap => vec![(PrimName::TranslateWrap, Params::TranslateWrap(1, 0))],
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
