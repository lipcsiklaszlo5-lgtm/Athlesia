#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
SEARCH_DIR = os.path.join(PROJECT, "crates", "athlesia-search")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. lib.rs bővítése: általános pontozófüggvényes A* keresés
lib_path = os.path.join(SEARCH_DIR, "src", "lib.rs")
content = pathlib.Path(lib_path).read_text()

if "pub fn a_star_search_with_score" not in content:
    score_code = r'''
/// A* keresés általános pontozófüggvénnyel.
/// A `score_fn` paraméter: (program, aktuális_grid, cél_grid, mélység) -> prioritás.
/// Kisebb érték = jobb (mint az A* f-score).
/// Ez lehetővé teszi, hogy a MetaLearner súlyait is figyelembe vegyük.
pub fn a_star_search_with_score<F>(
    input: &Grid,
    target: &Grid,
    max_depth: usize,
    score_fn: F,
) -> Option<Program>
where
    F: Fn(&Program, &Grid, &Grid, usize) -> usize,
{
    use std::collections::BinaryHeap;
    use std::cmp::Ordering;

    #[derive(Debug, Clone)]
    struct Node {
        program: Program,
        grid: Grid,
        depth: usize,
        f_score: usize,
    }

    impl PartialEq for Node {
        fn eq(&self, other: &Self) -> bool {
            self.f_score == other.f_score && self.program == other.program
        }
    }
    impl Eq for Node {}

    impl PartialOrd for Node {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for Node {
        fn cmp(&self, other: &Self) -> Ordering {
            other.f_score.cmp(&self.f_score)
        }
    }

    let mut heap = BinaryHeap::new();
    let mut initial_budget = Budget { max_steps: 0 };
    let initial_grid = run_program(&vec![], input, &mut initial_budget)
        .unwrap_or_else(|_| input.clone());

    let initial_program = vec![];
    let initial_score = score_fn(&initial_program, &initial_grid, target, 0);
    heap.push(Node {
        program: initial_program,
        grid: initial_grid,
        depth: 0,
        f_score: initial_score,
    });

    while let Some(node) = heap.pop() {
        if node.grid == *target {
            return Some(node.program);
        }
        if node.depth >= max_depth {
            continue;
        }

        for (prim, params) in candidate_primitives() {
            let mut new_program = node.program.clone();
            new_program.push((prim, params));
            let mut b = Budget { max_steps: new_program.len() as u64 };
            if let Ok(new_grid) = run_program(&new_program, input, &mut b) {
                let new_depth = node.depth + 1;
                let new_score = score_fn(&new_program, &new_grid, target, new_depth);
                heap.push(Node {
                    program: new_program,
                    grid: new_grid,
                    depth: new_depth,
                    f_score: new_score,
                });
            }
        }
    }
    None
}
'''
    content += score_code
    write_file(lib_path, content)
    print("[INFO] a_star_search_with_score hozzáadva.")

# 2. Teszt hozzáadása
test_content = r'''
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
'''
write_file(os.path.join(SEARCH_DIR, "tests", "score_search_test.rs"), test_content)
print("[INFO] Score-based search teszt hozzáadva.")

# 3. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-search"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Search score-based tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Search score-based tesztek zöldek.")

# 4. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add score-based A* search for MetaLearner integration"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
