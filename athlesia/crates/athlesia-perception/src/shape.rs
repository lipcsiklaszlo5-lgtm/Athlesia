
use crate::{GameObject, bounding_box};

/// Az objektum mérete cellában.
pub fn cell_count(obj: &GameObject) -> usize {
    obj.cells.len()
}

/// Az objektum befoglaló téglalapjának szélessége és magassága.
pub fn bbox_dimensions(obj: &GameObject) -> (usize, usize) {
    let (min_x, min_y, max_x, max_y) = bounding_box(obj);
    ((max_x - min_x + 1) as usize, (max_y - min_y + 1) as usize)
}

/// Mennyire tölti ki az alakzat a befoglaló téglalapját? 1.0 = tömör.
pub fn fill_ratio(obj: &GameObject) -> f32 {
    let (w, h) = bbox_dimensions(obj);
    if w == 0 || h == 0 {
        return 0.0;
    }
    obj.cells.len() as f32 / (w * h) as f32
}

/// Mennyire vonalszerű az alakzat?
/// 0.0 = kompakt (pl. négyzet), 1.0 = tökéletes egyenes.
/// A cellakoordináták kovarianciamátrixának sajátérték-arányából számolva.
pub fn linearity(obj: &GameObject) -> f32 {
    let n = obj.cells.len() as f64;
    if n == 0.0 {
        return 0.0;
    }

    // Centroid
    let sum_x: f64 = obj.cells.iter().map(|c| c.x as f64).sum();
    let sum_y: f64 = obj.cells.iter().map(|c| c.y as f64).sum();
    let cx = sum_x / n;
    let cy = sum_y / n;

    // Kovariancia-mátrix elemei
    let mut var_x = 0.0;
    let mut var_y = 0.0;
    let mut cov_xy = 0.0;
    for c in &obj.cells {
        let dx = c.x as f64 - cx;
        let dy = c.y as f64 - cy;
        var_x += dx * dx;
        var_y += dy * dy;
        cov_xy += dx * dy;
    }
    var_x /= n;
    var_y /= n;
    cov_xy /= n;

    // Sajátértékek zárt képlettel
    let trace = var_x + var_y;
    let det = var_x * var_y - cov_xy * cov_xy;
    let disc = (trace * trace / 4.0) - det;
    if disc < 0.0 {
        return 0.0; // numerikus hiba esetén semleges
    }
    let sqrt_disc = disc.sqrt();
    let lambda1 = trace / 2.0 + sqrt_disc;
    let lambda2 = trace / 2.0 - sqrt_disc;

    if lambda1 <= 0.0 {
        return 0.0;
    }

    // linearitás = 1 - (kisebb/nagyobb)
    // Ha lambda2 kicsi lambda1-hez képest, a pontok egy egyenes mentén helyezkednek el.
    let ratio = lambda2 / lambda1; // 0.0 és 1.0 között
    (1.0 - ratio) as f32
}
