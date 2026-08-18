
use athlesia_verifier::{Verifier, ResidualStructure};
use athlesia_types::{Grid, PrimName, Params, Program};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn report_exact_match() {
    let v = Verifier::new();
    let program: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];
    let input = build_grid([
        [1,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
    ]);
    let target = build_grid([
        [0,1,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
    ]);

    let report = v.report(&program, &input, &target);
    assert!(report.exact);
    assert_eq!(report.residual, ResidualStructure::None);
    assert_eq!(report.pixel_accuracy, 1.0);
}

#[test]
fn report_rejects_mismatch() {
    let v = Verifier::new();
    let program: Program = vec![(PrimName::ReflectH, Params::None)];
    let input = build_grid([
        [1,2,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
    ]);
    let target = build_grid([
        [2,1,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
    ]);

    let report = v.report(&program, &input, &target);
    assert!(!report.exact);
    assert!(report.pixel_accuracy < 1.0);
    assert!(report.pixel_accuracy > 0.0);
    assert_eq!(report.residual, ResidualStructure::Unknown);
}
