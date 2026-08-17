
use athlesia_search::search;
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn finds_single_step_translate() {
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

    let program = search(&input, &target, 1);
    assert!(program.is_some());
}

#[test]
fn finds_two_step_program() {
    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    // Két eltolás jobbra: (1,0) + (1,0) => (2,0)
    let target = build_grid([
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let program = search(&input, &target, 2);
    assert!(program.is_some());
}

#[test]
fn returns_none_when_no_solution() {
    let input = build_grid([[0; 5]; 5]);
    let target = build_grid([
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
    ]);
    let program = search(&input, &target, 2);
    assert!(program.is_none());
}
