
use athlesia_synthesis::{synthesize, PrimitiveTemplate};
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn synthesizes_translate_right() {
    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let target = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let program = synthesize(&input, &target, &[
        PrimitiveTemplate::Translate,
        PrimitiveTemplate::ReflectH,
        PrimitiveTemplate::ReflectV,
        PrimitiveTemplate::Rotate90,
        PrimitiveTemplate::Recolor,
    ]);

    assert!(program.is_some());
}

#[test]
fn synthesizes_recolor() {
    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let target = build_grid([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let program = synthesize(&input, &target, &[
        PrimitiveTemplate::Recolor,
        PrimitiveTemplate::Translate,
    ]);

    assert!(program.is_some());
}

#[test]
fn fails_when_no_program_solves() {
    let input = build_grid([[0; 5]; 5]);
    let target = build_grid([
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
    ]);

    let program = synthesize(&input, &target, &[
        PrimitiveTemplate::Translate,
        PrimitiveTemplate::ReflectH,
    ]);

    assert!(program.is_none());
}
