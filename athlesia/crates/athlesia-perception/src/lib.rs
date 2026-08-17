
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
