
use athlesia_types::{Color, Coord, Grid};

pub mod shape;
pub mod holes;

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
