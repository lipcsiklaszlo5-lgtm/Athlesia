
use std::collections::{HashSet, HashMap};
use crate::{GameObject, bounding_box};

/// Az objektum által teljesen körülzárt, összefüggő háttérüregek száma.
pub fn hole_count(obj: &GameObject) -> u8 {
    hole_sizes(obj).len() as u8
}

/// Az egyes lyukak mérete cellákban.
pub fn hole_sizes(obj: &GameObject) -> Vec<usize> {
    let cells: HashSet<(i8, i8)> = obj.cells.iter().map(|c| (c.x, c.y)).collect();
    let (min_x, min_y, max_x, max_y) = bounding_box(obj);

    // 1. A bbox-on belüli, nem-objektum cellák közül a bbox szegélyéről induló
    //    flood-fill elérhető cellái a "kültér" (outside). Ami nem elérhető, az lyuk.
    let mut outside: HashSet<(i8, i8)> = HashSet::new();
    let mut stack: Vec<(i8, i8)> = Vec::new();

    // Bbox szegélyén lévő nem-objektum cellákból indulunk.
    for y in min_y..=max_y {
        for x in [min_x, max_x] {
            let key = (x, y);
            if !cells.contains(&key) && !outside.contains(&key) {
                outside.insert(key);
                stack.push(key);
            }
        }
    }
    for x in min_x..=max_x {
        for y in [min_y, max_y] {
            let key = (x, y);
            if !cells.contains(&key) && !outside.contains(&key) {
                outside.insert(key);
                stack.push(key);
            }
        }
    }

    while let Some((x, y)) = stack.pop() {
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = x + dx;
            let ny = y + dy;
            if nx < min_x || nx > max_x || ny < min_y || ny > max_y {
                continue;
            }
            let key = (nx, ny);
            if !cells.contains(&key) && !outside.contains(&key) {
                outside.insert(key);
                stack.push(key);
            }
        }
    }

    // 2. A nem objektum és nem outside cellák lyukak.
    let mut hole_cells: HashSet<(i8, i8)> = HashSet::new();
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let key = (x, y);
            if !cells.contains(&key) && !outside.contains(&key) {
                hole_cells.insert(key);
            }
        }
    }

    // 3. Összefüggő komponensek keresése a lyukcellákon.
    let mut sizes = Vec::new();
    let mut visited: HashSet<(i8, i8)> = HashSet::new();

    for &seed in &hole_cells {
        if visited.contains(&seed) {
            continue;
        }
        let mut comp_size = 0usize;
        let mut comp_stack = vec![seed];
        visited.insert(seed);
        while let Some((x, y)) = comp_stack.pop() {
            comp_size += 1;
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = x + dx;
                let ny = y + dy;
                if nx < min_x || nx > max_x || ny < min_y || ny > max_y {
                    continue;
                }
                let key = (nx, ny);
                if hole_cells.contains(&key) && !visited.contains(&key) {
                    visited.insert(key);
                    comp_stack.push(key);
                }
            }
        }
        sizes.push(comp_size);
    }

    sizes
}

/// A legmélyebb lyuk mélysége az objektum falától.
/// A lyukcellákból, amelyeknek van objektum-szomszédja, BFS-t indítunk
/// a lyukcellákon, és a maximális elért távolságot adjuk vissza.
pub fn max_hole_depth(obj: &GameObject) -> u8 {
    let cells: HashSet<(i8, i8)> = obj.cells.iter().map(|c| (c.x, c.y)).collect();
    let (min_x, min_y, max_x, max_y) = bounding_box(obj);

    // Lyukcellák meghatározása ugyanúgy, mint a hole_sizes-ban.
    let mut outside: HashSet<(i8, i8)> = HashSet::new();
    let mut stack: Vec<(i8, i8)> = Vec::new();

    for y in min_y..=max_y {
        for x in [min_x, max_x] {
            let key = (x, y);
            if !cells.contains(&key) && !outside.contains(&key) {
                outside.insert(key);
                stack.push(key);
            }
        }
    }
    for x in min_x..=max_x {
        for y in [min_y, max_y] {
            let key = (x, y);
            if !cells.contains(&key) && !outside.contains(&key) {
                outside.insert(key);
                stack.push(key);
            }
        }
    }

    while let Some((x, y)) = stack.pop() {
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = x + dx;
            let ny = y + dy;
            if nx < min_x || nx > max_x || ny < min_y || ny > max_y {
                continue;
            }
            let key = (nx, ny);
            if !cells.contains(&key) && !outside.contains(&key) {
                outside.insert(key);
                stack.push(key);
            }
        }
    }

    let hole_cells: HashSet<(i8, i8)> = (min_y..=max_y)
        .flat_map(|y| (min_x..=max_x).map(move |x| (x, y)))
        .filter(|key| !cells.contains(key) && !outside.contains(key))
        .collect();

    if hole_cells.is_empty() {
        return 0;
    }

    // Források: lyukcellák, amelyeknek van objektum-szomszédja.
    let mut sources: Vec<(i8, i8)> = Vec::new();
    for &(x, y) in &hole_cells {
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = x + dx;
            let ny = y + dy;
            if nx < min_x || nx > max_x || ny < min_y || ny > max_y {
                continue;
            }
            if cells.contains(&(nx, ny)) {
                sources.push((x, y));
                break;
            }
        }
    }

    // BFS a lyukcellákon
    let mut max_depth = 0u8;
    let mut dist: HashMap<(i8, i8), u8> = HashMap::new();
    let mut queue: Vec<(i8, i8)> = Vec::new();

    for &src in &sources {
        if !dist.contains_key(&src) {
            dist.insert(src, 0);
            queue.push(src);
        }
    }

    while let Some((x, y)) = queue.pop() {
        let d = dist[&(x, y)];
        if d > max_depth {
            max_depth = d;
        }
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = x + dx;
            let ny = y + dy;
            if nx < min_x || nx > max_x || ny < min_y || ny > max_y {
                continue;
            }
            let key = (nx, ny);
            if hole_cells.contains(&key) && !dist.contains_key(&key) {
                dist.insert(key, d + 1);
                queue.push(key);
            }
        }
    }

    max_depth
}
