
use athlesia_abstraction::AbstractionEngine;
use athlesia_knowledge::KnowledgeBase;
use athlesia_types::{PrimName, Params, Program, Color};

#[test]
fn extracts_frequent_two_step_macro() {
    let mut kb = KnowledgeBase::new();

    let translate: (PrimName, Params) = (PrimName::Translate, Params::Translate(1, 0));
    let recolor: (PrimName, Params) = (PrimName::Recolor, Params::Recolor([Color(1), Color(0), Color(2), Color(3)]));

    // A gyakori kétlépéses minta: translate + recolor
    let pattern: Program = vec![translate, recolor];

    // Hét megoldott program, mindegyik tartalmazza ezt a mintát,
    // de különböző további lépésekkel.
    let solved = vec![
        pattern.clone(),
        vec![translate, recolor, (PrimName::ReflectH, Params::None)],
        vec![(PrimName::Rotate90, Params::None), translate, recolor],
        pattern.clone(),
        vec![translate, recolor, (PrimName::ReflectV, Params::None)],
        pattern.clone(),
        vec![(PrimName::Translate, Params::Translate(0, 1)), translate, recolor],
    ];

    let added = AbstractionEngine::extract_macros(&solved, &mut kb, 5);
    assert!(added >= 1, "Legalább egy makrót hozzá kell adni, hozzáadva: {}", added);

    // A kétlépéses mintának szerepelnie kell a makrók között
    let has_pattern = kb.get_all_macros().iter().any(|m| m.program == pattern);
    assert!(has_pattern, "A gyakori kétlépéses mintát makróként kell tárolni");
}

#[test]
fn does_not_extract_infrequent_two_step_macro() {
    let mut kb = KnowledgeBase::new();

    let a: Program = vec![
        (PrimName::Translate, Params::Translate(1, 0)),
        (PrimName::Recolor, Params::Recolor([Color(1), Color(0), Color(2), Color(3)])),
    ];
    let b: Program = vec![
        (PrimName::ReflectH, Params::None),
        (PrimName::Rotate90, Params::None),
    ];

    let solved = vec![a.clone(), b.clone()];

    let added = AbstractionEngine::extract_macros(&solved, &mut kb, 2);
    assert_eq!(added, 0, "Nem szabad makrót hozzáadni, mert nincs elég gyakori minta");
}
