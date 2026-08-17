
use crate::Grid;

/// Egy adott (px, py) periódus mennyire illik a rácsra.
/// Azt vizsgálja, hogy a rácson belül minden (r,c) és (r+py, c+px) cella
/// színe milyen arányban egyezik meg.
pub fn periodicity_score(grid: &Grid, period: (usize, usize)) -> f32 {
    let (px, py) = period;
    if px == 0 || py == 0 {
        return 1.0; // nincs eltolás, mindig egyezik
    }
    if px >= grid.width as usize || py >= grid.height as usize {
        return 0.0;
    }

    let mut total = 0usize;
    let mut matching = 0usize;

    for y in 0..(grid.height as usize - py) {
        for x in 0..(grid.width as usize - px) {
            let idx1 = y * grid.width as usize + x;
            let idx2 = (y + py) * grid.width as usize + (x + px);
            total += 1;
            if grid.cells[idx1] == grid.cells[idx2] {
                matching += 1;
            }
        }
    }

    if total == 0 {
        1.0
    } else {
        matching as f32 / total as f32
    }
}

/// A legkisebb periódus, amely a megadott küszöb feletti pontszámot ad.
/// Brute-force keresés az összes (px, py) kombinációra,
/// a legkisebb px*py szorzatú találatot visszaadva.
pub fn detect_periodicity(grid: &Grid, min_score: f32) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize)> = None;
    let mut best_area = usize::MAX;

    for py in 1..=grid.height as usize / 2 {
        for px in 1..=grid.width as usize / 2 {
            let score = periodicity_score(grid, (px, py));
            if score >= min_score {
                let area = px * py;
                if area < best_area {
                    best_area = area;
                    best = Some((px, py));
                }
            }
        }
    }

    best
}
