
use athlesia_knowledge::{KnowledgeBase, ChangeKind};
use athlesia_types::{PrimName, Params, Program};

#[test]
fn add_primitive_increases_version_and_archives() {
    let mut kb = KnowledgeBase::new();
    kb.add_primitive(PrimName::Translate);

    assert_eq!(kb.version, 1);
    assert_eq!(kb.primitives.len(), 1);
    assert_eq!(kb.archive.len(), 1);
    assert!(matches!(&kb.archive[0].change, ChangeKind::AddPrimitive(_)));
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
fn add_concept_stores_macro_refs_and_archives() {
    let mut kb = KnowledgeBase::new();
    kb.add_macro("macro1".to_string(), vec![(PrimName::Translate, Params::Translate(1,0))]);
    kb.add_macro("macro2".to_string(), vec![(PrimName::Rotate90, Params::None)]);

    // A két makró id-je: 0 és 1
    kb.add_concept("motion".to_string(), vec![0, 1]);

    assert_eq!(kb.version, 3); // 1 primitív? nem, csak macro-k: 1. macro -> version 1, 2. macro -> version 2, fogalom -> version 3
    assert_eq!(kb.concepts.len(), 1);
    assert!(kb.get_concept_by_name("motion").is_some());
    assert_eq!(kb.archive.len(), 3);
    assert!(matches!(&kb.archive[2].change, ChangeKind::AddConcept { name } if name == "motion"));
}

#[test]
fn do_not_duplicate_primitive() {
    let mut kb = KnowledgeBase::new();
    kb.add_primitive(PrimName::Translate);
    kb.add_primitive(PrimName::Translate);

    assert_eq!(kb.primitives.len(), 1);
    assert_eq!(kb.version, 1);
}
