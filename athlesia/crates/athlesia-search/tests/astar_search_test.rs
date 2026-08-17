
use athlesia_search::{a_star_search, search};
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn astar_finds_same_solution_as_dfs() {
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
    let astar_solution = a_star_search(&input, &target, 2);

    assert!(dfs_solution.is_some());
    assert!(astar_solution.is_some());
    assert_eq!(dfs_solution, astar_solution);
}

#[test]
fn astar_returns_none_when_no_solution() {
    let input = build_grid([[0; 5]; 5]);
    let target = build_grid([
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
    ]);
    let solution = a_star_search(&input, &target, 2);
    assert!(solution.is_none());
}
