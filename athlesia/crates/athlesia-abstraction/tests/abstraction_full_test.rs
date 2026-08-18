
use athlesia_abstraction::AbstractionEngine;
use athlesia_knowledge::KnowledgeBase;
use athlesia_types::{PrimName, Params, Program, Color};

#[test]
fn extracts_frequent_subsequence_as_macro() {
    let mut kb = KnowledgeBase::new();

    let translate: (PrimName, Params) = (PrimName::Translate, Params::Translate(1, 0));
    let recolor: (PrimName, Params) = (PrimName::Recolor, Params::Recolor([
        Color(1), Color(0), Color(2), Color(3), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)
    ]));

    // Gyakori részsorozat: translate.clone() + recolor.clone()
    let pattern: Program = vec![translate.clone(), recolor.clone()];

    let solved = vec![
        pattern.clone(),
        vec![translate.clone(), recolor.clone(), (PrimName::ReflectH, Params::None)],
        vec![(PrimName::Rotate90, Params::None), translate.clone(), recolor.clone()],
        pattern.clone(),
        vec![translate.clone(), recolor.clone(), (PrimName::ReflectV, Params::None)],
    ];

    let added = AbstractionEngine::extract_macros(&solved, &mut kb, 4);
    assert!(added >= 1, "Legalább egy makrót hozzá kell adni");

    let has_pattern = kb.get_all_macros().iter().any(|m| m.program == pattern);
    assert!(has_pattern, "A gyakori részsorozatot makróként kell tárolni");
}

#[test]
fn anti_unify_finds_common_subsequence() {
    let a: Program = vec![
        (PrimName::Translate, Params::Translate(1, 0)),
        (PrimName::Rotate90, Params::None),
        (PrimName::Recolor, Params::Recolor([
            Color(1), Color(0), Color(2), Color(3), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)
        ])),
    ];
    let b: Program = vec![
        (PrimName::Rotate90, Params::None),
        (PrimName::Translate, Params::Translate(1, 0)),
        (PrimName::Recolor, Params::Recolor([
            Color(1), Color(0), Color(2), Color(3), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)
        ])),
    ];

    let lcs = AbstractionEngine::anti_unify(&a, &b);
    // A leghosszabb közös részsorozat: [Translate, Recolor] vagy [Rotate90, Recolor]
    assert!(lcs.len() == 2);
    assert_eq!(lcs[1], a[2]);
}

#[test]
fn does_not_extract_infrequent_patterns() {
    let mut kb = KnowledgeBase::new();

    let a: Program = vec![(PrimName::ReflectH, Params::None)];
    let b: Program = vec![(PrimName::ReflectV, Params::None)];

    let solved = vec![a, b];

    let added = AbstractionEngine::extract_macros(&solved, &mut kb, 2);
    assert_eq!(added, 0);
}
