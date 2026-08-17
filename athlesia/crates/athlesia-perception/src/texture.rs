
use crate::{Grid, Color};

/// A rács leggyakoribb (domináns) színe.
pub fn background_color(grid: &Grid) -> Color {
    let mut hist = [0usize; 10];
    for cell in &grid.cells {
        if cell.0 < 10 {
            hist[cell.0 as usize] += 1;
        }
    }
    let mut best_color = 0u8;
    let mut best_count = 0usize;
    for (color, &count) in hist.iter().enumerate() {
        if count > best_count {
            best_count = count;
            best_color = color as u8;
        }
    }
    Color(best_color)
}

/// A háttérszínnel fedett terület aránya 0.0 és 1.0 között.
pub fn empty_area_ratio(grid: &Grid) -> f32 {
    let bg = background_color(grid);
    let total = (grid.width as usize) * (grid.height as usize);
    if total == 0 {
        return 0.0;
    }
    let bg_count = grid.cells.iter().filter(|c| **c == bg).count();
    bg_count as f32 / total as f32
}

/// Durva textúra-térkép: blokkonkénti nem-háttér arány.
/// A rácsot `block_size × block_size` blokkokra osztjuk, és minden blokkra
/// kiszámoljuk a nem-háttér cellák arányát. A kimenet sorfolytonos `Vec<f32>`.
pub fn density_grid(grid: &Grid, block_size: usize) -> Vec<f32> {
    let bg = background_color(grid);
    let mut result = Vec::new();

    let mut y = 0usize;
    while y < grid.height as usize {
        let mut x = 0usize;
        while x < grid.width as usize {
            let block_h = block_size.min(grid.height as usize - y);
            let block_w = block_size.min(grid.width as usize - x);

            let mut total = 0usize;
            let mut non_bg = 0usize;
            for by in 0..block_h {
                for bx in 0..block_w {
                    let idx = (y + by) * grid.width as usize + (x + bx);
                    if grid.cells[idx] != bg {
                        non_bg += 1;
                    }
                    total += 1;
                }
            }
            let ratio = if total > 0 { non_bg as f32 / total as f32 } else { 0.0 };
            result.push(ratio);

            x += block_size;
        }
        y += block_size;
    }

    result
}
