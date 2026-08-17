
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
