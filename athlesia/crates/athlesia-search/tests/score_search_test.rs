
use athlesia_search::a_star_search_with_score;
use athlesia_types::{Grid, Program};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn score_based_astar_finds_solution() {
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

    // Egyszerű pontozó: hány cella egyezik a targettel, plusz a mélység
    let score = |prog: &Program, grid: &Grid, target: &Grid, depth: usize| -> usize {
        let mut match_count = 0;
        for i in 0..grid.cells.len() {
            for j in 0..grid.cells[0].len() {
                if grid.cells[i][j] == target.cells[i][j] {
                    match_count += 1;
                }
            }
        }
        let mismatch = 25 - match_count;
        mismatch + depth
    };

    let solution = a_star_search_with_score(&input, &target, 3, score);
    assert!(solution.is_some());
    assert!(!solution.unwrap().is_empty());
}
