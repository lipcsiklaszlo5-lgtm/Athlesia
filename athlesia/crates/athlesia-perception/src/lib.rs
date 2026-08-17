
use athlesia_types::{Color, Coord, Grid, GRID_SIZE};

#[derive(Debug, Clone)]
pub struct GameObject {
    pub id: u64,
    pub color: Color,
    pub cells: Vec<Coord>,
}

/// Flood-fill alapú összefüggő komponens keresés.
/// 0 szín = háttér (nem objektum).
pub fn segment(grid: &Grid) -> Vec<GameObject> {
    let mut visited = [[false; GRID_SIZE]; GRID_SIZE];
    let mut objects = Vec::new();
    let mut next_id = 0u64;

    for i in 0..GRID_SIZE {
        for j in 0..GRID_SIZE {
            if visited[i][j] {
                continue;
            }
            let color = grid.cells[i][j];
            if color == 0 {
                visited[i][j] = true;
                continue;
            }

            let mut stack = vec![(i as i8, j as i8)];
            let mut cells = Vec::new();
            visited[i][j] = true;

            while let Some((x, y)) = stack.pop() {
                cells.push(Coord { x, y });

                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = x + dx;
                    let ny = y + dy;
                    if nx >= 0 && nx < GRID_SIZE as i8 && ny >= 0 && ny < GRID_SIZE as i8 {
                        let ni = nx as usize;
                        let nj = ny as usize;
                        if !visited[ni][nj] && grid.cells[ni][nj] == color {
                            visited[ni][nj] = true;
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

/// Két objektum akkor érintkezik, ha van olyan cellájuk,
/// amelyek egymás mellett vannak (Manhattan-távolság = 1).
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


/// Bounding box: (min_x, min_y, max_x, max_y) a cellákból.
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

/// Centroid: az objektum celláinak átlagos pozíciója.
pub fn centroid(obj: &GameObject) -> (f64, f64) {
    let n = obj.cells.len() as f64;
    let sum_x: f64 = obj.cells.iter().map(|c| c.x as f64).sum();
    let sum_y: f64 = obj.cells.iter().map(|c| c.y as f64).sum();
    (sum_x / n, sum_y / n)
}

/// Centroidok euklideszi távolsága.
pub fn distance_between(a: &GameObject, b: &GameObject) -> f64 {
    let (ax, ay) = centroid(a);
    let (bx, by) = centroid(b);
    ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt()
}

/// Relatív irány A-tól B-hez: (dx, dy) irányvektor, komponensenként -1, 0 vagy 1.
/// Például ha B jobbra és lefelé van A-hoz képest, akkor (1, 1).
pub fn relative_direction(a: &GameObject, b: &GameObject) -> (i8, i8) {
    let (ax, ay) = centroid(a);
    let (bx, by) = centroid(b);
    let dx = (bx - ax).signum() as i8;
    let dy = (by - ay).signum() as i8;
    (dx, dy)
}

/// Igaz, ha A befoglaló téglalapja teljesen tartalmazza B-ét, és A != B.
pub fn contains(a: &GameObject, b: &GameObject) -> bool {
    if a.id == b.id {
        return false;
    }
    let (amin_x, amin_y, amax_x, amax_y) = bounding_box(a);
    let (bmin_x, bmin_y, bmax_x, bmax_y) = bounding_box(b);
    amin_x <= bmin_x && amin_y <= bmin_y && amax_x >= bmax_x && amax_y >= bmax_y
}
