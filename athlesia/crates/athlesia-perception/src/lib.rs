
use std::collections::{HashMap, HashSet};
use athlesia_types::{Color, Coord, Grid};

pub mod shape;
pub mod holes;
pub mod symmetry;
pub mod texture;
pub mod pattern;

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


/// Két objektum méretének aránya.
pub fn relative_size(a: &GameObject, b: &GameObject) -> f32 {
    let size_a = a.cells.len() as f32;
    let size_b = b.cells.len() as f32;
    if size_b == 0.0 {
        return 0.0;
    }
    size_a / size_b
}

/// Normalizált irányvektor a és b centroidja között, a rács átlójával osztva.
pub fn relative_offset(a: &GameObject, b: &GameObject, grid: &Grid) -> (f32, f32) {
    let (ax, ay) = centroid(a);
    let (bx, by) = centroid(b);
    let diag = ((grid.width as f64).powi(2) + (grid.height as f64).powi(2)).sqrt() as f32;
    if diag == 0.0 {
        return (0.0, 0.0);
    }
    (((bx - ax) as f32 / diag), ((by - ay) as f32 / diag))
}

/// Két objektum bbox-a metszi-e egymást sorban.
pub fn shares_row(a: &GameObject, b: &GameObject) -> bool {
    let (_, min_y_a, _, max_y_a) = bounding_box(a);
    let (_, min_y_b, _, max_y_b) = bounding_box(b);
    min_y_a <= max_y_b && min_y_b <= max_y_a
}

/// Két objektum bbox-a metszi-e egymást oszlopban.
pub fn shares_col(a: &GameObject, b: &GameObject) -> bool {
    let (min_x_a, _, max_x_a, _) = bounding_box(a);
    let (min_x_b, _, max_x_b, _) = bounding_box(b);
    min_x_a <= max_x_b && min_x_b <= max_x_a
}

/// A teljes rács színeloszlása.
pub fn color_histogram(grid: &Grid) -> [usize; 10] {
    let mut hist = [0usize; 10];
    for cell in &grid.cells {
        if cell.0 < 10 {
            hist[cell.0 as usize] += 1;
        }
    }
    hist
}

/// Azonos színű objektumok csoportosítása.
pub fn group_objects_by_color(objects: &[GameObject]) -> HashMap<Color, Vec<u64>> {
    let mut map: HashMap<Color, Vec<u64>> = HashMap::new();
    for obj in objects {
        map.entry(obj.color).or_default().push(obj.id);
    }
    map
}

/// Térben közeli objektumok klaszterezése.
pub fn cluster_objects_by_proximity(objects: &[GameObject], max_dist: f32) -> Vec<Vec<u64>> {
    let mut parent: Vec<usize> = (0..objects.len()).collect();

    fn find(parent: &mut [usize], x: usize) -> usize {
        if parent[x] != x {
            parent[x] = find(parent, parent[x]);
        }
        parent[x]
    }

    fn union(parent: &mut [usize], x: usize, y: usize) {
        let px = find(parent, x);
        let py = find(parent, y);
        if px != py {
            parent[py] = px;
        }
    }

    for i in 0..objects.len() {
        for j in (i + 1)..objects.len() {
            if distance_between(&objects[i], &objects[j]) <= max_dist as f64 {
                union(&mut parent, i, j);
            }
        }
    }

    let mut clusters: HashMap<usize, Vec<u64>> = HashMap::new();
    for (idx, obj) in objects.iter().enumerate() {
        let root = find(&mut parent, idx);
        clusters.entry(root).or_default().push(obj.id);
    }
    clusters.into_values().collect()
}


/// Az előző és aktuális képkocka közötti változások.
#[derive(Debug, Clone, Default)]
pub struct FrameDelta {
    pub added: Vec<Coord>,
    pub removed: Vec<Coord>,
    pub changed: Vec<Coord>,
}

/// Két grid közötti cellánkénti különbség.
pub fn diff_grids(prev: &Grid, current: &Grid) -> FrameDelta {
    let mut delta = FrameDelta::default();
    if prev.width != current.width || prev.height != current.height {
        // Méretváltozás esetén egyszerűen minden nem-nulla cella added, és minden prev cella removed
        for y in 0..current.height as i8 {
            for x in 0..current.width as i8 {
                if current.get(x, y).map_or(false, |c| c != Color(0)) {
                    delta.added.push(Coord { x, y });
                }
            }
        }
        for y in 0..prev.height as i8 {
            for x in 0..prev.width as i8 {
                if prev.get(x, y).map_or(false, |c| c != Color(0)) {
                    delta.removed.push(Coord { x, y });
                }
            }
        }
        return delta;
    }

    for y in 0..current.height as i8 {
        for x in 0..current.width as i8 {
            let old = prev.get(x, y);
            let new = current.get(x, y);
            match (old, new) {
                (Some(o), Some(n)) if o != n => delta.changed.push(Coord { x, y }),
                (None, Some(_)) => delta.added.push(Coord { x, y }),
                (Some(_), None) => delta.removed.push(Coord { x, y }),
                _ => {}
            }
        }
    }
    delta
}

/// Objektumgráf: az objektumok listája, és a köztük lévő relációk.
#[derive(Debug, Clone, Default)]
pub struct ObjectGraph {
    pub objects: Vec<GameObject>,
    pub touching_pairs: Vec<(usize, usize)>,
    pub contains_pairs: Vec<(usize, usize)>,
}

/// A percepciós csővezeték kimenete: az objektumgráf és a frame-delta.
#[derive(Debug, Clone, Default)]
pub struct PerceptionOutput {
    pub graph: ObjectGraph,
    pub delta: FrameDelta,
}

/// Objektum-ujjlenyomat: forgatás- és tükrözés-invariáns leírás.
pub fn shape_fingerprint(obj: &GameObject) -> (Color, Vec<(i8, i8)>) {
    let (min_x, min_y, _, _) = bounding_box(obj);
    let mut rel: Vec<(i8, i8)> = obj.cells.iter().map(|c| (c.x - min_x, c.y - min_y)).collect();
    rel.sort_unstable();
    (obj.color, rel)
}

/// Két frame objektumainak párosítása ujjlenyomat alapján.
/// Visszaadja a matched párokat (prev_index, current_index).
pub fn track_objects(prev_objects: &[GameObject], current_objects: &[GameObject]) -> Vec<(usize, usize)> {
    let mut matches = Vec::new();
    let mut used_current = HashSet::new();

    for (pi, p_obj) in prev_objects.iter().enumerate() {
        let fp_p = shape_fingerprint(p_obj);
        for (ci, c_obj) in current_objects.iter().enumerate() {
            if used_current.contains(&ci) {
                continue;
            }
            if shape_fingerprint(c_obj) == fp_p {
                matches.push((pi, ci));
                used_current.insert(ci);
                break;
            }
        }
    }
    matches
}

/// A jelenlegi gridből teljes PerceptionOutput-ot készít.
pub fn perceive(prev: Option<&Grid>, current: &Grid) -> PerceptionOutput {
    let objects = segment(current);
    let mut graph = ObjectGraph {
        objects,
        touching_pairs: Vec::new(),
        contains_pairs: Vec::new(),
    };

    // Relációk kiszámítása
    for i in 0..graph.objects.len() {
        for j in (i + 1)..graph.objects.len() {
            if touches(&graph.objects[i], &graph.objects[j]) {
                graph.touching_pairs.push((i, j));
            }
            if contains(&graph.objects[i], &graph.objects[j]) || contains(&graph.objects[j], &graph.objects[i]) {
                graph.contains_pairs.push((i, j));
            }
        }
    }

    let delta = match prev {
        Some(p) => diff_grids(p, current),
        None => FrameDelta::default(),
    };

    PerceptionOutput { graph, delta }
}
