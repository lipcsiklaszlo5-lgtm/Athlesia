#!/usr/bin/env python3
import os, re, sys, pathlib
from pathlib import Path

# Segédfüggvény
def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Típusok frissítése dinamikus Grid-re
types_lib = r'''
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Color(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Coord {
    pub x: i8,
    pub y: i8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grid {
    pub width: u8,
    pub height: u8,
    pub cells: Vec<Color>,
}

impl Grid {
    pub fn new(width: u8, height: u8) -> Self {
        Grid {
            width,
            height,
            cells: vec![Color(0); (width as usize) * (height as usize)],
        }
    }

    /// 5×5-ös teszt-adatokból hoz létre Grid-et.
    pub fn from_5x5(rows: [[u8; 5]; 5]) -> Self {
        let mut cells = Vec::with_capacity(25);
        for row in &rows {
            for &v in row {
                cells.push(Color(v));
            }
        }
        Grid { width: 5, height: 5, cells }
    }

    pub fn set(&mut self, x: i8, y: i8, color: Color) {
        if x >= 0 && x < self.width as i8 && y >= 0 && y < self.height as i8 {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            self.cells[idx] = color;
        }
    }

    pub fn get(&self, x: i8, y: i8) -> Option<Color> {
        if x >= 0 && x < self.width as i8 && y >= 0 && y < self.height as i8 {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            Some(self.cells[idx])
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimName {
    Translate,
    ReflectH,
    ReflectV,
    Rotate90,
    Recolor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Params {
    None,
    Translate(i8, i8),
    Recolor([Color; 4]),
}

pub type Program = Vec<(PrimName, Params)>;

#[derive(Debug, Clone, Copy)]
pub struct Budget {
    pub max_steps: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecError {
    BudgetExceeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Action {
    pub prim: PrimName,
    pub params: Params,
}
'''
write_file("crates/athlesia-types/src/lib.rs", types_lib)
print("[1] Types frissítve.")

# 2. Executor frissítése: GRID_SIZE eltávolítás, clone használata
executor_lib = r'''
use athlesia_types::{Grid, PrimName, Params, Program, Budget, ExecError};

pub use athlesia_types::Budget as PublicBudget;

pub fn apply_primitive(grid: &Grid, name: &PrimName, params: &Params) -> Grid {
    let mut new_grid = Grid::new(grid.width, grid.height);

    match name {
        PrimName::Translate => {
            if let Params::Translate(dx, dy) = params {
                let dx = *dx as i8;
                let dy = *dy as i8;
                for y in 0..grid.height as i8 {
                    for x in 0..grid.width as i8 {
                        let (nx, ny) = (x + dx, y + dy);
                        if nx >= 0 && nx < grid.width as i8 && ny >= 0 && ny < grid.height as i8 {
                            if let Some(color) = grid.get(x, y) {
                                new_grid.set(nx, ny, color);
                            }
                        }
                    }
                }
            }
        }
        PrimName::ReflectH => {
            for y in 0..grid.height as i8 {
                for x in 0..grid.width as i8 {
                    if let Some(color) = grid.get(x, y) {
                        new_grid.set(grid.width as i8 - 1 - x, y, color);
                    }
                }
            }
        }
        PrimName::ReflectV => {
            for y in 0..grid.height as i8 {
                for x in 0..grid.width as i8 {
                    if let Some(color) = grid.get(x, y) {
                        new_grid.set(x, grid.height as i8 - 1 - y, color);
                    }
                }
            }
        }
        PrimName::Rotate90 => {
            for y in 0..grid.height as i8 {
                for x in 0..grid.width as i8 {
                    if let Some(color) = grid.get(x, y) {
                        new_grid.set(grid.height as i8 - 1 - y, x, color);
                    }
                }
            }
        }
        PrimName::Recolor => {
            if let Params::Recolor(perm) = params {
                for y in 0..grid.height as i8 {
                    for x in 0..grid.width as i8 {
                        if let Some(color) = grid.get(x, y) {
                            let new_color = perm[color.0 as usize];
                            new_grid.set(x, y, Color(new_color));
                        }
                    }
                }
            }
        }
    }

    new_grid
}

pub fn run_program(program: &Program, input: &Grid, budget: &mut Budget) -> Result<Grid, ExecError> {
    let mut current = input.clone();

    for (name, params) in program {
        if budget.max_steps == 0 {
            return Err(ExecError::BudgetExceeded);
        }
        current = apply_primitive(&current, name, params);
        budget.max_steps -= 1;
    }

    Ok(current)
}
'''
# Az executorban van egy régi `pub use athlesia_types::Budget;` sor, ami mostantól nem kell.
executor_lib = executor_lib.replace("pub use athlesia_types::Budget as PublicBudget;", "")
write_file("crates/athlesia-executor/src/lib.rs", executor_lib)
print("[2] Executor frissítve.")

# 3. Perception frissítése: dinamikus méretek
perception_lib = r'''
use athlesia_types::{Color, Coord, Grid};

#[derive(Debug, Clone)]
pub struct GameObject {
    pub id: u64,
    pub color: Color,
    pub cells: Vec<Coord>,
}

pub fn segment(grid: &Grid) -> Vec<GameObject> {
    let mut visited = vec![false; (grid.width as usize) * (grid.height as usize)];
    let mut objects = Vec::new();
    let mut next_id = 0u64;

    for y in 0..grid.height as i8 {
        for x in 0..grid.width as i8 {
            let idx = (y as usize) * (grid.width as usize) + (x as usize);
            if visited[idx] {
                continue;
            }
            let color = match grid.get(x, y) {
                Some(c) if c != Color(0) => c,
                _ => {
                    visited[idx] = true;
                    continue;
                }
            };

            let mut stack = vec![(x, y)];
            let mut cells = Vec::new();
            visited[idx] = true;

            while let Some((cx, cy)) = stack.pop() {
                cells.push(Coord { x: cx, y: cy });

                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = cx + dx;
                    let ny = cy + dy;
                    if nx >= 0 && nx < grid.width as i8 && ny >= 0 && ny < grid.height as i8 {
                        let nidx = (ny as usize) * (grid.width as usize) + (nx as usize);
                        if !visited[nidx] && grid.get(nx, ny) == Some(color) {
                            visited[nidx] = true;
                            stack.push((nx, ny));
                        }
                    }
                }
            }

            objects.push(GameObject {
                id: next_id,
                color,
                cells,
            });
            next_id += 1;
        }
    }

    objects
}

pub fn touches(a: &GameObject, b: &GameObject) -> bool {
    for ca in &a.cells {
        for cb in &b.cells {
            if (ca.x - cb.x).abs() + (ca.y - cb.y).abs() == 1 {
                return true;
            }
        }
    }
    false
}

pub fn bounding_box(obj: &GameObject) -> (i8, i8, i8, i8) {
    let mut min_x = i8::MAX;
    let mut min_y = i8::MAX;
    let mut max_x = i8::MIN;
    let mut max_y = i8::MIN;
    for c in &obj.cells {
        if c.x < min_x { min_x = c.x; }
        if c.y < min_y { min_y = c.y; }
        if c.x > max_x { max_x = c.x; }
        if c.y > max_y { max_y = c.y; }
    }
    (min_x, min_y, max_x, max_y)
}

pub fn centroid(obj: &GameObject) -> (f64, f64) {
    let n = obj.cells.len() as f64;
    let sum_x: f64 = obj.cells.iter().map(|c| c.x as f64).sum();
    let sum_y: f64 = obj.cells.iter().map(|c| c.y as f64).sum();
    (sum_x / n, sum_y / n)
}

pub fn distance_between(a: &GameObject, b: &GameObject) -> f64 {
    let (ax, ay) = centroid(a);
    let (bx, by) = centroid(b);
    ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt()
}

pub fn relative_direction(a: &GameObject, b: &GameObject) -> (i8, i8) {
    let (ax, ay) = centroid(a);
    let (bx, by) = centroid(b);
    let dx = (bx - ax).signum() as i8;
    let dy = (by - ay).signum() as i8;
    (dx, dy)
}

pub fn contains(a: &GameObject, b: &GameObject) -> bool {
    if a.id == b.id {
        return false;
    }
    let (amin_x, amin_y, amax_x, amax_y) = bounding_box(a);
    let (bmin_x, bmin_y, bmax_x, bmax_y) = bounding_box(b);
    amin_x <= bmin_x && amin_y <= bmin_y && amax_x >= bmax_x && amax_y >= bmax_y
}
'''
write_file("crates/athlesia-perception/src/lib.rs", perception_lib)
print("[3] Perception frissítve.")

# 4. Features frissítése: dinamikus méretek
features_lib = r'''
use std::collections::HashMap;
use athlesia_types::{Grid, Color};
use athlesia_perception::{segment, touches, contains, distance_between, relative_direction};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FeatureVector {
    pub object_count: u8,
    pub color_counts: [u8; 4],
    pub touching_pairs: u8,
    pub has_hole: bool,
    pub symmetric_h: bool,
    pub symmetric_v: bool,
    pub contains_pairs: u8,
    pub min_distance_category: u8,
    pub dominant_direction: (i8, i8),
}

pub fn extract_features(grid: &Grid) -> FeatureVector {
    let objects = segment(grid);
    let object_count = objects.len() as u8;

    let mut color_counts = [0u8; 4];
    for &cell in &grid.cells {
        if cell.0 < 4 {
            color_counts[cell.0 as usize] += 1;
        }
    }

    let mut touching_pairs = 0;
    for i in 0..objects.len() {
        for j in (i + 1)..objects.len() {
            if touches(&objects[i], &objects[j]) {
                touching_pairs += 1;
            }
        }
    }

    let mut contains_pairs = 0;
    for i in 0..objects.len() {
        for j in (i + 1)..objects.len() {
            if contains(&objects[i], &objects[j]) || contains(&objects[j], &objects[i]) {
                contains_pairs += 1;
            }
        }
    }

    let mut min_distance_category = 0u8;
    if objects.len() >= 2 {
        let mut any_touching = false;
        let mut min_dist = f64::MAX;
        for i in 0..objects.len() {
            for j in (i + 1)..objects.len() {
                if touches(&objects[i], &objects[j]) {
                    any_touching = true;
                }
                let d = distance_between(&objects[i], &objects[j]);
                if d < min_dist {
                    min_dist = d;
                }
            }
        }
        if any_touching {
            min_distance_category = 1;
        } else if min_dist <= 2.0 {
            min_distance_category = 2;
        } else {
            min_distance_category = 3;
        }
    }

    let mut dir_counts: HashMap<(i8, i8), usize> = HashMap::new();
    for i in 0..objects.len() {
        for j in (i + 1)..objects.len() {
            let dir = relative_direction(&objects[i], &objects[j]);
            *dir_counts.entry(dir).or_insert(0) += 1;
        }
    }
    let dominant_direction = dir_counts
        .into_iter()
        .max_by_key(|(dir, count)| (*count, std::cmp::Reverse(*dir)))
        .map(|(dir, _)| dir)
        .unwrap_or((0, 0));

    let has_hole = detect_hole(grid);
    let (symmetric_h, symmetric_v) = bounding_box_symmetry(grid);

    FeatureVector {
        object_count,
        color_counts,
        touching_pairs,
        has_hole,
        symmetric_h,
        symmetric_v,
        contains_pairs,
        min_distance_category,
        dominant_direction,
    }
}

fn detect_hole(grid: &Grid) -> bool {
    let width = grid.width as usize;
    let height = grid.height as usize;
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if grid.cells[idx] == Color(0) {
                let up = y > 0 && grid.cells[(y - 1) * width + x] != Color(0);
                let down = y + 1 < height && grid.cells[(y + 1) * width + x] != Color(0);
                let left = x > 0 && grid.cells[y * width + x - 1] != Color(0);
                let right = x + 1 < width && grid.cells[y * width + x + 1] != Color(0);
                if up && down && left && right {
                    return true;
                }
            }
        }
    }
    false
}

fn bounding_box_symmetry(grid: &Grid) -> (bool, bool) {
    let width = grid.width as usize;
    let height = grid.height as usize;

    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut has_object = false;

    for y in 0..height {
        for x in 0..width {
            if grid.cells[y * width + x] != Color(0) {
                has_object = true;
                if x < min_x { min_x = x; }
                if x > max_x { max_x = x; }
                if y < min_y { min_y = y; }
                if y > max_y { max_y = y; }
            }
        }
    }

    if !has_object {
        return (true, true);
    }

    let bbox_width = max_x - min_x + 1;
    let bbox_height = max_y - min_y + 1;

    let mut sym_h = true;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let mir_x = max_x - (x - min_x);
            if grid.cells[y * width + x] != grid.cells[y * width + mir_x] {
                sym_h = false;
                break;
            }
        }
        if !sym_h { break; }
    }

    let mut sym_v = true;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let mir_y = max_y - (y - min_y);
            if grid.cells[y * width + x] != grid.cells[mir_y * width + x] {
                sym_v = false;
                break;
            }
        }
        if !sym_v { break; }
    }

    (sym_h, sym_v)
}
'''
write_file("crates/athlesia-features/src/lib.rs", features_lib)
print("[4] Features frissítve.")

# 5. Search frissítése: dinamikus méretek
search_lib = r'''
use athlesia_types::{Grid, PrimName, Params, Program, Budget, GRID_SIZE, Color};
use athlesia_executor::run_program;

fn candidate_primitives() -> Vec<(PrimName, Params)> {
    let mut v = Vec::new();
    for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0), (0, 0)] {
        v.push((PrimName::Translate, Params::Translate(dx, dy)));
    }
    v.push((PrimName::ReflectH, Params::None));
    v.push((PrimName::ReflectV, Params::None));
    v.push((PrimName::Rotate90, Params::None));
    for perm in [
        [1, 0, 2, 3],
        [2, 1, 0, 3],
        [3, 2, 1, 0],
        [1, 2, 3, 0],
        [0, 1, 2, 3],
    ] {
        let arr: [Color; 4] = [
            Color(perm[0]),
            Color(perm[1]),
            Color(perm[2]),
            Color(perm[3]),
        ];
        v.push((PrimName::Recolor, Params::Recolor(arr)));
    }
    v
}

pub fn search(input: &Grid, target: &Grid, max_depth: usize) -> Option<Program> {
    fn dfs(
        input: &Grid,
        target: &Grid,
        max_depth: usize,
        depth: usize,
        current: &mut Program,
    ) -> Option<Program> {
        if depth == max_depth {
            let mut budget = Budget { max_steps: depth as u64 };
            if let Ok(output) = run_program(current, input, &mut budget) {
                if output == *target {
                    return Some(current.clone());
                }
            }
            return None;
        }

        for (prim, params) in candidate_primitives() {
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

pub fn beam_search(input: &Grid, target: &Grid, max_depth: usize, beam_width: usize) -> Option<Program> {
    let mut beam: Vec<(Program, Grid)> = Vec::new();
    beam.push((Vec::new(), input.clone()));

    for _depth in 0..max_depth {
        let mut next_beam: Vec<(Program, Grid)> = Vec::new();

        for (program, _current_grid) in &beam {
            for (prim, params) in candidate_primitives() {
                let mut new_program = program.clone();
                new_program.push((prim, params));
                let mut budget = Budget { max_steps: new_program.len() as u64 };
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

        for (prim, params) in candidate_primitives() {
            let mut new_program = node.program.clone();
            new_program.push((prim, params));
            let mut budget = Budget { max_steps: new_program.len() as u64 };
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

        for (prim, params) in candidate_primitives() {
            let mut new_program = node.program.clone();
            new_program.push((prim, params));
            let mut budget = Budget { max_steps: new_program.len() as u64 };
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

fn score_grid(grid: &Grid, target: &Grid) -> usize {
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
'''
write_file("crates/athlesia-search/src/lib.rs", search_lib)
print("[5] Search frissítve.")

# 6. Golden test grid_from_vec frissítése
golden_path = Path("crates/athlesia-executor/tests/golden_test.rs")
golden_content = golden_path.read_text()
old_golden = '''fn grid_from_vec(v: &[Vec<u8>]) -> Grid {
    let mut cells = [[0u8; GRID_SIZE]; GRID_SIZE];
    for i in 0..GRID_SIZE { for j in 0..GRID_SIZE { cells[i][j] = v[i][j]; } }
    Grid { cells }
}'''
new_golden = '''fn grid_from_vec(v: &[Vec<u8>]) -> Grid {
    let height = v.len() as u8;
    let width = if height > 0 { v[0].len() as u8 } else { 0 };
    let mut cells = Vec::with_capacity((width as usize) * (height as usize));
    for row in v {
        for &cell in row {
            cells.push(Color(cell));
        }
    }
    Grid { width, height, cells }
}'''
if old_golden in golden_content:
    golden_content = golden_content.replace(old_golden, new_golden)
    golden_path.write_text(golden_content)
    print("[6] Golden test grid_from_vec frissítve.")
else:
    print("[WARN] Nem találtam a golden test grid_from_vec blokkot.")

# 7. Tesztfájlok build_grid cseréje
for test_file in Path("crates").rglob("tests/*.rs"):
    if test_file.name == "golden_test.rs":
        continue  # már kezeltük
    content = test_file.read_text()
    old_build = '''fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}'''
    new_build = '''fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}'''
    if old_build in content:
        content = content.replace(old_build, new_build)
        test_file.write_text(content)
        print(f"[7] build_grid frissítve: {test_file}")
    else:
        # Lehet, hogy a build_grid más formátumú, például `Grid { cells: rows }` nélkül.
        if "fn build_grid" in content:
            print(f"[WARN] build_grid definíció nem egyezik: {test_file}")

print("Fájlok módosítva, tesztek futtatása...")

# 8. Cargo test workspace
result = subprocess.run(["cargo", "test", "--workspace"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Workspace tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Workspace tesztek zöldek.")
