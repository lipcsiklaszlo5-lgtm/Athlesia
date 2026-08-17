
use athlesia_hypothesis::{HypothesisProposer, StaticProposer};
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
