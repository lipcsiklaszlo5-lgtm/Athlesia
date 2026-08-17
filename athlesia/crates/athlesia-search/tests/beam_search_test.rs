
use athlesia_search::{search, beam_search};
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn beam_search_finds_same_solution_as_dfs() {
    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let target = build_grid([
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let dfs_solution = search(&input, &target, 2);
    let beam_solution = beam_search(&input, &target, 2, 5);

    assert!(dfs_solution.is_some());
    assert!(beam_solution.is_some());
    assert_eq!(dfs_solution, beam_solution);
}
