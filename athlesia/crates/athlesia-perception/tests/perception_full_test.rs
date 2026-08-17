
use athlesia_perception::{
    segment, diff_grids, shape_fingerprint, track_objects, perceive
};
use athlesia_types::{Grid, Color};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn diff_grids_detects_change() {
    let prev = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let cur = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let delta = diff_grids(&prev, &cur);
    assert_eq!(delta.changed.len(), 2); // a (0,0) és (0,1) is változott
}

#[test]
fn shape_fingerprint_is_translation_invariant() {
    let g1 = build_grid([
        [1, 1, 0, 0, 0],
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let g2 = build_grid([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 1, 1, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let objs1 = segment(&g1);
    let objs2 = segment(&g2);
    assert_eq!(objs1.len(), 1);
    assert_eq!(objs2.len(), 1);

    let fp1 = shape_fingerprint(&objs1[0]);
    let fp2 = shape_fingerprint(&objs2[0]);
    assert_eq!(fp1, fp2);
}

#[test]
fn track_objects_matches_by_fingerprint() {
    let g1 = build_grid([
        [1, 1, 0, 0, 0],
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let g2 = build_grid([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 1, 1, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let prev_objs = segment(&g1);
    let cur_objs = segment(&g2);
    let matches = track_objects(&prev_objs, &cur_objs);
    assert_eq!(matches, vec![(0, 0)]);
}

#[test]
fn perceive_builds_graph_and_delta() {
    let prev = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let cur = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let output = perceive(Some(&prev), &cur);
    assert_eq!(output.graph.objects.len(), 1);
    assert_eq!(output.delta.changed.len(), 2);
}
