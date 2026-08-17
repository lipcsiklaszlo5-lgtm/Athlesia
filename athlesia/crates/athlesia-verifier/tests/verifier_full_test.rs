
use athlesia_verifier::{Verifier, VerificationResult};
use athlesia_types::{Grid, PrimName, Params, Program, Color};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn accepts_correct_program() {
    let v = Verifier::new();
    let program: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];

    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let expected = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let result = v.verify(&program, &vec![(input, expected)]);
    assert_eq!(result, VerificationResult::Accept);
}

#[test]
fn rejects_wrong_program() {
    let v = Verifier::new();
    let program: Program = vec![(PrimName::ReflectH, Params::None)];

    let input = build_grid([
        [1, 2, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let expected = build_grid([
        [2, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let result = v.verify(&program, &vec![(input, expected)]);
    assert_eq!(result, VerificationResult::Reject);
}

#[test]
fn returns_inconclusive_for_empty_examples() {
    let v = Verifier::new();
    let program: Program = vec![(PrimName::Recolor, Params::Recolor([
        Color(1), Color(0), Color(2), Color(3), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)
    ]))];
    let result = v.verify(&program, &Vec::new());
    assert_eq!(result, VerificationResult::Inconclusive);
}

#[test]
fn verify_hypothesis_checks_full_history() {
    let v = Verifier::new();
    let program: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];

    let history = vec![
        (
            build_grid([[1,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]),
            build_grid([[0,1,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]),
        ),
        (
            build_grid([[1,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]),
            build_grid([[0,1,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]),
        ),
    ];

    let result = v.verify_hypothesis(&program, &history);
    assert_eq!(result, VerificationResult::Accept);
}

#[test]
fn verify_equivalence_detects_same_program() {
    let v = Verifier::new();
    let p1: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];
    let p2: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];

    let examples = vec![
        (build_grid([[1,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]),
         build_grid([[0,1,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]])),
    ];

    let result = v.verify_equivalence(&p1, &p2, &examples);
    assert_eq!(result, VerificationResult::Accept);
}

#[test]
fn verify_equivalence_rejects_different_programs() {
    let v = Verifier::new();
    let p1: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];
    let p2: Program = vec![(PrimName::ReflectH, Params::None)];

    let examples = vec![
        (build_grid([[1,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]),
         build_grid([[0,1,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]])),
    ];

    let result = v.verify_equivalence(&p1, &p2, &examples);
    assert_eq!(result, VerificationResult::Reject);
}
