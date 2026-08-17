
use athlesia_verifier::{Verifier, VerificationResult};
use athlesia_types::{Grid, PrimName, Params, Program};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
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
    let program: Program = vec![(PrimName::Recolor, Params::Recolor([1, 0, 2, 3]))];
    let result = v.verify(&program, &[]);
    assert_eq!(result, VerificationResult::Inconclusive);
}
