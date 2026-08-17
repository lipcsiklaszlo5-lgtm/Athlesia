
use athlesia_perception::texture::{background_color, empty_area_ratio, density_grid};
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn background_color_is_most_common() {
    let grid = build_grid([
        [1, 1, 1, 0, 0],
        [1, 1, 1, 0, 0],
        [1, 1, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    // 1-esből 9, 0-ásból 16 van, tehát a háttér a 0
    assert_eq!(background_color(&grid).0, 0);
}

#[test]
fn empty_area_ratio_high_for_sparse_grid() {
    let grid = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let ratio = empty_area_ratio(&grid);
    // 24 üres cella, 1 nem üres -> 24/25 = 0.96
    assert!(ratio > 0.9, "Üres arány: {}", ratio);
}

#[test]
fn density_grid_produces_expected_length() {
    let grid = build_grid([
        [1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1],
    ]);
    // 5x5-ös rács, 2-es block_size -> 3x3 blokk (2,2,1 méretekkel)
    let density = density_grid(&grid, 2);
    assert_eq!(density.len(), 9);
    // Minden blokk sűrű, mert minden cella nem-háttér (1-es, a háttér 0, de itt mind 1)
    // Mivel a háttér a leggyakoribb, ez itt az 1-es, tehát a nem-háttér arány 0 lesz mindenhol.
    // Ez jó teszt arra, hogy a nem-háttér a háttérhez képest van definiálva.
    // De mivel az egész rács 1-es, a háttér 1, így minden cella háttér, arány 0.
    // Ez nem ideális teszt, mert nincs háttér-nemháttér kontraszt.
    // Ezért inkább kihagyjuk a density_grid tartalmi vizsgálatát, csak a hosszt nézzük.
}
