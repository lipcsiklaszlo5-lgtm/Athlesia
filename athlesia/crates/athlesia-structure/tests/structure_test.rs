
use athlesia_structure::{TargetDecomposer, TransformId};
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
fn decompose_2x2_to_4x4_all_identity() {
    let input = grid_from_rows(vec![
        vec![1, 2],
        vec![3, 4],
    ]);
    let target = grid_from_rows(vec![
        vec![1,2,1,2],
        vec![3,4,3,4],
        vec![1,2,1,2],
        vec![3,4,3,4],
    ]);

    let decomposer = TargetDecomposer;
    let meta = decomposer.decompose(&input, &target).expect("Dekompozíció nem található");
    assert_eq!(meta.rows, 2);
    assert_eq!(meta.cols, 2);
    assert!(meta.cells.iter().all(|t| *t == Some(TransformId::Identity)));
}

#[test]
fn decompose_2x2_to_4x4_with_rotations() {
    let input = grid_from_rows(vec![
        vec![1, 2],
        vec![3, 4],
    ]);
    let target = grid_from_rows(vec![
        vec![1,2,2,4],
        vec![3,4,1,3],
        vec![2,1,4,3],
        vec![4,3,2,1],
    ]);

    let decomposer = TargetDecomposer;
    let meta = decomposer.decompose(&input, &target).expect("Dekompozíció nem található");
    assert_eq!(meta.rows, 2);
    assert_eq!(meta.cols, 2);

    let expected = vec![
        Some(TransformId::Identity),
        Some(TransformId::Rot90),
        Some(TransformId::ReflectH),
        Some(TransformId::Rot180),
    ];
    assert_eq!(meta.cells, expected);
}

#[test]
fn no_decomposition_when_dimensions_not_divisible() {
    let input = grid_from_rows(vec![vec![1, 2]]);
    let target = grid_from_rows(vec![vec![1, 2, 3]]);

    let decomposer = TargetDecomposer;
    assert!(decomposer.decompose(&input, &target).is_none());
}
