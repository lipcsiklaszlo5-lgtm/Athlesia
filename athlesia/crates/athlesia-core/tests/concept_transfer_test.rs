
use athlesia_core::CoreEngine;
use athlesia_types::{Grid, Color};

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
fn concept_transfer_composes_known_programs() {
    let mut engine = CoreEngine::new();

    // Tanuljuk meg a ReflectH-t
    let input_refl = grid_from_rows(vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
    ]);
    let target_refl = grid_from_rows(vec![
        vec![3, 2, 1],
        vec![6, 5, 4],
        vec![9, 8, 7],
    ]);
    let (_, steps_refl) = engine.solve_with_steps(&input_refl, &target_refl);
    assert!(steps_refl > 0, "ReflectH tanulásnak kellett történnie");

    // Tanuljuk meg a Translate(1,0)-t
    let input_trans = grid_from_rows(vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
    ]);
    let target_trans = grid_from_rows(vec![
        vec![0, 1, 2],
        vec![0, 4, 5],
        vec![0, 7, 8],
    ]);
    let (_, steps_trans) = engine.solve_with_steps(&input_trans, &target_trans);
    assert!(steps_trans > 0, "Translate tanulásnak kellett történnie");

    // Most jön a kombinált feladat: ReflectH + Translate(1,0)
    let input_comb = grid_from_rows(vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
    ]);
    let target_comb = grid_from_rows(vec![
        vec![0, 3, 2],
        vec![0, 6, 5],
        vec![0, 9, 8],
    ]);

    let (result, steps_comb) = engine.solve_with_steps(&input_comb, &target_comb);
    assert!(result.is_some(), "A kombinált feladatot meg kellett oldani");
    let program = result.unwrap();
    // A megoldásnak két primitív kombinációjából kell állnia: ReflectH és Translate
    assert_eq!(program.len(), 2, "A programnak két lépésből kell állnia, de {} lépésből áll", program.len());
    assert!(
        program.iter().any(|(prim, _)| *prim == athlesia_types::PrimName::ReflectH)
        && program.iter().any(|(prim, _)| *prim == athlesia_types::PrimName::Translate),
        "A programnak ReflectH-t és Translate-et is tartalmaznia kell"
    );
    // A kompozíciós megoldás nem igényelhet teljes keresést; lépésszáma alacsony marad.
    assert!(steps_comb < 20, "A kompozíciós megoldás túl sok lépést igényelt: {}", steps_comb);
}
