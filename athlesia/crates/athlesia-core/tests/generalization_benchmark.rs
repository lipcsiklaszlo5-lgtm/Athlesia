
use athlesia_core::CoreEngine;
use athlesia_types::{Grid, Color};

/// Egyszerű rács létrehozása sorvektorokból.
fn grid_from_rows(rows: Vec<Vec<u8>>) -> Grid {
    let height = rows.len() as u8;
    let width = if height > 0 { rows[0].len() as u8 } else { 0 };
    let mut cells = Vec::new();
    for row in &rows {
        for &c in row {
            cells.push(Color(c));
        }
    }
    Grid { width, height, cells }
}

#[test]
fn generalization_reduces_search_cost_for_reflect_h_then_translate() {
    let mut engine = CoreEngine::new();

    // Első feladat: ReflectH + Translate(1,0) (jobbra tolás).
    let input1 = grid_from_rows(vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
    ]);
    let target1 = grid_from_rows(vec![
        vec![0, 3, 2],
        vec![0, 6, 5],
        vec![0, 9, 8],
    ]);

    let (result1, steps1) = engine.solve_with_steps(&input1, &target1);
    assert!(result1.is_some(), "Az első feladatot meg kellett oldani");
    assert!(steps1 > 1, "Az első feladat keresést igényel, steps1={}", steps1);

    // Második feladat: ugyanaz a szabály, nagyobb méretben.
    let input2 = grid_from_rows(vec![
        vec![1, 0, 2, 0, 3],
        vec![4, 5, 0, 6, 7],
        vec![0, 8, 9, 0, 1],
        vec![2, 3, 0, 4, 5],
        vec![6, 0, 7, 8, 0],
    ]);
    // ReflectH + jobbra tolás
    let target2 = grid_from_rows(vec![
        vec![0, 3, 0, 2, 0],
        vec![0, 7, 6, 0, 5],
        vec![0, 1, 0, 9, 8],
        vec![0, 5, 4, 0, 3],
        vec![0, 0, 8, 7, 0],
    ]);

    let (result2, steps2) = engine.solve_with_steps(&input2, &target2);
    assert!(result2.is_some(), "A második feladatot is meg kellett oldani");
    assert!(
        steps2 < steps1,
        "A második feladatnak kevesebb lépéssel kell megoldódnia: steps1={}, steps2={}",
        steps1,
        steps2
    );
}
