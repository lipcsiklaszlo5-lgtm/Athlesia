
use athlesia_types::{Grid, PrimName, Params, Program, Budget, Color};
use athlesia_executor::run_program;

/// Keresési stratégiák, amelyeket a Search Engine támogat.
pub enum SearchStrategy {
    Dfs,
    Beam { width: usize },
    AStar,
    AStarWithScore,
}

/// Keresési prioritás: minél alacsonyabb, annál jobb.
pub type Priority = usize;

/// A keresőmotor által használt általános interfész.
pub trait SearchEngine {
    fn search(
        &self,
        input: &Grid,
        target: &Grid,
        max_depth: usize,
        strategy: SearchStrategy,
    ) -> Option<Program>;
}

/// A Manhattan Kernel alapértelmezett keresőmotorja.
pub struct DefaultSearchEngine;

impl SearchEngine for DefaultSearchEngine {
    fn search(
        &self,
        input: &Grid,
        target: &Grid,
        max_depth: usize,
        strategy: SearchStrategy,
    ) -> Option<Program> {
        match strategy {
            SearchStrategy::Dfs => search(input, target, max_depth),
            SearchStrategy::Beam { width } => beam_search(input, target, max_depth, width),
            SearchStrategy::AStar => a_star_search(input, target, max_depth),
            SearchStrategy::AStarWithScore => {
                let score_fn = |_: &Program, grid: &Grid, target: &Grid, depth: usize| -> usize {
                    let mut mismatch = 0usize;
                    for y in 0..grid.height as usize {
                        for x in 0..grid.width as usize {
                            if grid.cells[y * grid.width as usize + x] != target.cells[y * target.width as usize + x] {
                                mismatch += 1;
                            }
                        }
                    }
                    mismatch + depth
                };
                a_star_search_with_score(input, target, max_depth, score_fn)
            }
        }
    }
}

/// Primitívek listája, amelyeket a kereső kipróbál.
fn candidate_primitives(input: &Grid, target: &Grid) -> Vec<(PrimName, Params)> {
    let mut v = Vec::new();

    // Dimenzióváltó primitívek kikövetkeztetése a célméretből
    if input.width > 0 && input.height > 0
        && target.width % input.width == 0
        && target.height % input.height == 0
    {
        let w_ratio = target.width / input.width;
        let h_ratio = target.height / input.height;
        if w_ratio == h_ratio && w_ratio > 1 {
            let k = w_ratio as usize;
            v.push((PrimName::RepeatGrid, Params::RepeatGrid(k)));
            v.push((PrimName::Tile, Params::Tile(k)));
        }
    }

    for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0), (0, 0)] {
        v.push((PrimName::Translate, Params::Translate(dx, dy)));
    }

    v.push((PrimName::ReflectH, Params::None));
    v.push((PrimName::ReflectV, Params::None));
    v.push((PrimName::Rotate90, Params::None));
    v.push((PrimName::Rotate180, Params::None));
    v.push((PrimName::Rotate270, Params::None));

    v.push((PrimName::SwapColors, Params::SwapColors(1, 2)));
    v.push((PrimName::SwapColors, Params::SwapColors(1, 3)));
    v.push((PrimName::SwapColors, Params::SwapColors(2, 3)));

    v.push((PrimName::TranslateWrap, Params::TranslateWrap(1, 0)));
    v.push((PrimName::TranslateWrap, Params::TranslateWrap(0, 1)));
    v.push((PrimName::TranslateWrap, Params::TranslateWrap(-1, 0)));
    v.push((PrimName::TranslateWrap, Params::TranslateWrap(0, -1)));

    let identity: [Color; 10] = [
        Color(0), Color(1), Color(2), Color(3), Color(4),
        Color(5), Color(6), Color(7), Color(8), Color(9),
    ];
    v.push((PrimName::Recolor, Params::Recolor(identity)));

    let swap12: [Color; 10] = [
        Color(0), Color(2), Color(1), Color(3), Color(4),
        Color(5), Color(6), Color(7), Color(8), Color(9),
    ];
    v.push((PrimName::Recolor, Params::Recolor(swap12)));

    let swap13: [Color; 10] = [
        Color(0), Color(3), Color(2), Color(1), Color(4),
        Color(5), Color(6), Color(7), Color(8), Color(9),
    ];
    v.push((PrimName::Recolor, Params::Recolor(swap13)));

    v
}

/// Mélységi keresés.
pub fn search(input: &Grid, target: &Grid, max_depth: usize) -> Option<Program> {
    fn dfs(
        input: &Grid,
        target: &Grid,
        max_depth: usize,
        depth: usize,
        current: &mut Program,
    ) -> Option<Program> {
        if depth == max_depth {
            let mut budget = Budget { max_steps: depth as u64, max_depth: 100 };
            if let Ok(output) = run_program(current, input, &mut budget) {
                if output == *target {
                    return Some(current.clone());
                }
            }
            return None;
        }

        for (prim, params) in candidate_primitives(input, target) {
            current.push((prim, params));
            if let Some(found) = dfs(input, target, max_depth, depth + 1, current) {
                return Some(found);
            }
            current.pop();
        }
        None
    }

    for d in 1..=max_depth {
        let mut program = Vec::new();
        if let Some(p) = dfs(input, target, d, 0, &mut program) {
            return Some(p);
        }
    }
    None
}

/// Nyalábkeresés.
pub fn beam_search(input: &Grid, target: &Grid, max_depth: usize, beam_width: usize) -> Option<Program> {
    let mut beam: Vec<(Program, Grid)> = Vec::new();
    beam.push((Vec::new(), input.clone()));

    for _depth in 0..max_depth {
        let mut next_beam: Vec<(Program, Grid)> = Vec::new();

        for (program, _current_grid) in &beam {
            for (prim, params) in candidate_primitives(input, target) {
                let mut new_program = program.clone();
                new_program.push((prim, params));
                let mut budget = Budget { max_steps: new_program.len() as u64, max_depth: 100 };
                if let Ok(new_grid) = run_program(&new_program, input, &mut budget) {
                    next_beam.push((new_program, new_grid));
                }
            }
        }

        next_beam.sort_by(|a, b| {
            let score_a = score_grid(&a.1, target);
            let score_b = score_grid(&b.1, target);
            score_b.cmp(&score_a)
        });

        beam = next_beam.into_iter().take(beam_width).collect();

        for (program, grid) in &beam {
            if *grid == *target {
                return Some(program.clone());
            }
        }
    }
    None
}

/// A* keresés.
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
            other.f_score.cmp(&self.f_score)
        }
    }

    fn heuristic(grid: &Grid, target: &Grid) -> usize {
        let mut diff = 0;
        for i in 0..grid.height as usize {
            for j in 0..grid.width as usize {
                if grid.cells[i * grid.width as usize + j] != target.cells[i * target.width as usize + j] {
                    diff += 1;
                }
            }
        }
        diff
    }

    let mut heap = BinaryHeap::new();
    let initial_grid = input.clone();
    let initial_program = Vec::new();
    let initial_score = heuristic(&initial_grid, target);
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

        for (prim, params) in candidate_primitives(input, target) {
            let mut new_program = node.program.clone();
            new_program.push((prim, params));
            let mut budget = Budget { max_steps: new_program.len() as u64, max_depth: 100 };
            if let Ok(new_grid) = run_program(&new_program, input, &mut budget) {
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

/// A* keresés egyéni pontozófüggvénnyel.
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
    let initial_grid = input.clone();
    let initial_program = Vec::new();
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

        for (prim, params) in candidate_primitives(input, target) {
            let mut new_program = node.program.clone();
            new_program.push((prim, params));
            let mut budget = Budget { max_steps: new_program.len() as u64, max_depth: 100 };
            if let Ok(new_grid) = run_program(&new_program, input, &mut budget) {
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

/// Rács pontozása: hány cella egyezik a céllal.
fn score_grid(grid: &Grid, target: &Grid) -> usize {
    if grid.width != target.width || grid.height != target.height {
        return 0;
    }
    let mut score = 0;
    for i in 0..grid.height as usize {
        for j in 0..grid.width as usize {
            let idx = i * grid.width as usize + j;
            let tidx = i * target.width as usize + j;
            if grid.cells[idx] == target.cells[tidx] {
                score += 1;
            }
        }
    }
    score
}
