#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
SEARCH_DIR = os.path.join(PROJECT, "crates", "athlesia-search")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. lib.rs bővítése A* kereséssel
lib_path = os.path.join(SEARCH_DIR, "src", "lib.rs")
content = pathlib.Path(lib_path).read_text()

if "pub fn a_star_search" not in content:
    astar_code = r'''
/// A* keresés: a prioritás a megtett út hossza + a hátralévő becsült költség.
/// A heurisztika: az aktuális rács és a célrács közötti eltérő cellák száma.
/// Ez egy egyszerű, de hatékony alsó becslés (0, ha már elértük a célt).
pub fn a_star_search(input: &Grid, target: &Grid, max_depth: usize) -> Option<Program> {
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
            // Fordított sorrend, mert BinaryHeap a legnagyobbat veszi előre
            other.f_score.cmp(&self.f_score)
        }
    }

    // Heurisztika: hány cella tér el a célrácstól
    fn heuristic(grid: &Grid, target: &Grid) -> usize {
        let mut diff = 0;
        for i in 0..GRID_SIZE {
            for j in 0..GRID_SIZE {
                if grid.cells[i][j] != target.cells[i][j] {
                    diff += 1;
                }
            }
        }
        diff
    }

    let mut heap = BinaryHeap::new();
    let mut initial_budget = Budget { max_steps: 0 };
    let initial_grid = run_program(&vec![], input, &mut initial_budget)
        .unwrap_or_else(|_| input.clone());
    let initial_node = Node {
        program: vec![],
        grid: initial_grid.clone(),
        depth: 0,
        f_score: heuristic(&initial_grid, target),
    };
    heap.push(initial_node);

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
                let new_f = new_depth + heuristic(&new_grid, target);
                heap.push(Node {
                    program: new_program,
                    grid: new_grid,
                    depth: new_depth,
                    f_score: new_f,
                });
            }
        }
    }
    None
}
'''
    content += astar_code
    write_file(lib_path, content)
    print("[INFO] a_star_search hozzáadva.")

# 2. Teszt hozzáadása
test_content = r'''
use athlesia_search::{a_star_search, search};
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
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
'''
write_file(os.path.join(SEARCH_DIR, "tests", "astar_search_test.rs"), test_content)
print("[INFO] A* teszt hozzáadva.")

# 3. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-search"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Search A* tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Search A* tesztek zöldek.")

# 4. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add A* search to search engine"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
