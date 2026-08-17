
use athlesia_knowledge::{KnowledgeBase, ChangeKind};
use athlesia_types::{PrimName, Params, Program};

#[test]
fn add_primitive_increases_version_and_archives() {
    let mut kb = KnowledgeBase::new();
    assert_eq!(kb.version, 0);

    kb.add_primitive(PrimName::Translate);
    assert_eq!(kb.version, 1);
    assert_eq!(kb.primitives.len(), 1);
    assert_eq!(kb.archive.len(), 1);
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
fn do_not_duplicate_primitive() {
    let mut kb = KnowledgeBase::new();
    kb.add_primitive(PrimName::Translate);
    kb.add_primitive(PrimName::Translate);

    assert_eq!(kb.primitives.len(), 1);
    assert_eq!(kb.version, 1);
}

#[test]
fn change_kind_is_recorded_correctly() {
    let mut kb = KnowledgeBase::new();
    kb.add_primitive(PrimName::Rotate90);

    if let Some(entry) = kb.archive.last() {
        match &entry.change {
            ChangeKind::AddPrimitive(p) => assert_eq!(*p, PrimName::Rotate90),
            _ => panic!("Hibás ChangeKind"),
        }
    } else {
        panic!("Nincs bejegyzés az archívumban");
    }
}
