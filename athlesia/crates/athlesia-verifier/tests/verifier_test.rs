
use athlesia_verifier::{Verifier, VerificationResult};
use athlesia_types::{Grid, PrimName, Params, Program, Color};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn accepts_correct_program() {
    let v = Verifier;
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

    let result = v.verify(&program, &[(input, expected)]);
    assert_eq!(result, VerificationResult::Accept);
}

#[test]
fn rejects_wrong_program() {
    let v = Verifier;
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

    let result = v.verify(&program, &[(input, expected)]);
    assert_eq!(result, VerificationResult::Reject);
}

#[test]
fn returns_inconclusive_for_empty_examples() {
    let v = Verifier;
    let program: Program = vec![(PrimName::Recolor, Params::Recolor([Color(1), Color(0), Color(2), Color(3), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)]))];
    let result = v.verify(&program, &[]);
    assert_eq!(result, VerificationResult::Inconclusive);
}
