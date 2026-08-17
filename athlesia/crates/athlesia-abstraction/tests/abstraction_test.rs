
use athlesia_abstraction::AbstractionEngine;
use athlesia_knowledge::KnowledgeBase;
use athlesia_types::{PrimName, Params, Program};

#[test]
fn extracts_frequent_single_step_program() {
    let mut kb = KnowledgeBase::new();

    let translate: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];
    let reflect: Program = vec![(PrimName::ReflectH, Params::None)];

    // Négy megoldott program, ebből három ugyanaz a translate
    let solved = vec![
        translate.clone(),
        translate.clone(),
        translate.clone(),
        reflect.clone(),
    ];

    let added = AbstractionEngine::extract_macros(&solved, &mut kb, 3);
    assert_eq!(added, 1, "Egy makrót kell hozzáadni");
    assert_eq!(kb.get_all_macros().len(), 1);
    assert_eq!(kb.get_all_macros()[0].program, translate);
}

#[test]
fn does_not_extract_infrequent_program() {
    let mut kb = KnowledgeBase::new();

    let translate: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];
    let reflect: Program = vec![(PrimName::ReflectH, Params::None)];

    let solved = vec![translate.clone(), reflect.clone()];

    let added = AbstractionEngine::extract_macros(&solved, &mut kb, 2);
    assert_eq!(added, 0, "Nem szabad makrót hozzáadni, mert nincs elég gyakori minta");
    assert_eq!(kb.get_all_macros().len(), 0);
}

#[test]
fn respects_existing_macro() {
    let mut kb = KnowledgeBase::new();

    let translate: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];

    // Már létező makró
    kb.add_macro("existing_macro".to_string(), translate.clone());

    let solved = vec![translate.clone(), translate.clone(), translate.clone()];

    let added = AbstractionEngine::extract_macros(&solved, &mut kb, 2);
    assert_eq!(added, 0, "Nem szabad duplikátum makrót hozzáadni");
    assert_eq!(kb.get_all_macros().len(), 1);
}
