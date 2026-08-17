
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
