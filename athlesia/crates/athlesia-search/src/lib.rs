
use athlesia_types::{Grid, PrimName, Params, Program, Budget, GRID_SIZE};
use athlesia_executor::run_program;

/// Determinisztikus, korlátos mélységű programkeresés.
/// A cél: olyan programot találni, amely az inputból a cél gridet állítja elő.
/// A keresés a lehetséges primitívek kombinációit próbálja ki,
/// de nem az összeset, hanem egy rögzített, korlátozott paraméterhalmazt.
///
/// A keresés mélysége `max_depth` lépés. Minden lépésben minden primitív kipróbálható.
/// Determinisztikus, mert a primitívek listája és a bejárás sorrendje rögzített.

fn candidate_primitives() -> Vec<(PrimName, Params)> {
    let mut v = Vec::new();

    // Eltolások: 4 irány + identitás
    for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0), (0, 0)] {
        v.push((PrimName::Translate, Params::Translate(dx, dy)));
    }

    // Tükrözések
    v.push((PrimName::ReflectH, Params::None));
    v.push((PrimName::ReflectV, Params::None));

    // Forgás
    v.push((PrimName::Rotate90, Params::None));

    // Néhány színpermutáció
    for perm in [
        [1, 0, 2, 3],
        [2, 1, 0, 3],
        [3, 2, 1, 0],
        [1, 2, 3, 0],
        [0, 1, 2, 3],
    ] {
        v.push((PrimName::Recolor, Params::Recolor(perm)));
    }

    v
}

/// Rekurzív keresés: a `depth` hátralévő lépés számát jelzi.
/// A `current` a jelenlegi program, a `input` az eredeti rács, a `target` a cél.
fn dfs(
    input: &Grid,
    target: &Grid,
    max_depth: usize,
    depth: usize,
    current: &mut Program,
    budget: &mut Budget,
) -> Option<Program> {
    if depth == max_depth {
        // Kiértékeljük a teljes programot
        let mut b = Budget { max_steps: max_depth as u64 };
        if let Ok(output) = run_program(current, input, &mut b) {
            if output == *target {
                return Some(current.clone());
            }
        }
        return None;
    }

    for (prim, params) in candidate_primitives() {
        current.push((prim, params));
        if let Some(found) = dfs(input, target, max_depth, depth + 1, current, budget) {
            return Some(found);
        }
        current.pop();
    }
    None
}

/// Nyilvános kereső: iterál a mélységeken, és visszaadja az első találatot.
pub fn search(input: &Grid, target: &Grid, max_depth: usize) -> Option<Program> {
    for d in 1..=max_depth {
        let mut program = Vec::new();
        let mut budget = Budget { max_steps: d as u64 };
        if let Some(p) = dfs(input, target, d, 0, &mut program, &mut budget) {
            return Some(p);
        }
    }
    None
}


/// A beam search a lehetséges programteret szélességben járja be,
/// de egyszerre csak a legjobb `beam_width` jelöltet tartja meg.
/// A "jóság" mértéke most egyszerű: hány cella egyezik a cél-griddel.
/// Ez a jövőben a MetaLearner tanult súlyaira cserélhető.
pub fn beam_search(
    input: &Grid,
    target: &Grid,
    max_depth: usize,
    beam_width: usize,
) -> Option<Program> {
    // A jelöltek: (program, eddigi kimenet, pontszám)
    let mut beam: Vec<(Program, Grid)> = Vec::new();

    // Kezdeti üres program
    let initial_program = Vec::new();
    let mut budget = Budget { max_steps: 0 };
    let initial_grid = match run_program(&initial_program, input, &mut budget) {
        Ok(g) => g,
        Err(_) => input.clone(),
    };
    beam.push((initial_program, initial_grid));

    for _depth in 0..max_depth {
        let mut next_beam: Vec<(Program, Grid)> = Vec::new();

        for (program, _current_grid) in &beam {
            for (prim, params) in candidate_primitives() {
                let mut new_program = program.clone();
                new_program.push((prim, params));
                let mut b = Budget { max_steps: new_program.len() as u64 };
                if let Ok(new_grid) = run_program(&new_program, input, &mut b) {
                    next_beam.push((new_program, new_grid));
                }
            }
        }

        // Rendezés pontszám szerint: hány cella egyezik a target-tel
        next_beam.sort_by(|a, b| {
            let score_a = score_grid(&a.1, target);
            let score_b = score_grid(&b.1, target);
            score_b.cmp(&score_a)
        });

        // Csak a legjobb beam_width darab marad
        beam = next_beam.into_iter().take(beam_width).collect();

        // Ha valamelyik pontosan célba ért, visszaadjuk
        for (program, grid) in &beam {
            if *grid == *target {
                return Some(program.clone());
            }
        }
    }

    None
}

/// Pontszám: a targettel egyező cellák száma.
fn score_grid(grid: &Grid, target: &Grid) -> usize {
    let mut score = 0;
    for i in 0..GRID_SIZE {
        for j in 0..GRID_SIZE {
            if grid.cells[i][j] == target.cells[i][j] {
                score += 1;
            }
        }
    }
    score
}

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
