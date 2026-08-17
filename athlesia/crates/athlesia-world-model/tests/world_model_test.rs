use athlesia_world_model::{WorldModel, HypothesisStatus};
use athlesia_types::{Grid, PrimName, Params, Program};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn hypothesis_gets_evidence_for_correct_prediction() {
    let mut wm = WorldModel::new(build_grid([[0; 5]; 5]));

    // Program: Translate(1,0) - jobbra tolás
    let program: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];
    wm.add_hypothesis(program.clone());

    let prev = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let obs = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    wm.update(&prev, &obs);
    assert_eq!(wm.hypotheses[0].evidence_for, 1);
    assert_eq!(wm.hypotheses[0].evidence_against, 0);
    assert_eq!(wm.hypotheses[0].status, HypothesisStatus::Active);
}

#[test]
fn hypothesis_gets_evidence_against_wrong_prediction() {
    let mut wm = WorldModel::new(build_grid([[0; 5]; 5]));

    let program: Program = vec![(PrimName::ReflectH, Params::None)];
    wm.add_hypothesis(program.clone());

    let prev = build_grid([
        [1, 2, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let obs = prev.clone();

    wm.update(&prev, &obs);
    assert_eq!(wm.hypotheses[0].evidence_against, 1);
    assert_eq!(wm.hypotheses[0].status, HypothesisStatus::Falsified);
}

#[test]
fn multiple_updates_confirm_hypothesis() {
    let mut wm = WorldModel::new(build_grid([[0; 5]; 5]));

    // Identitás transzformáció: Translate(0,0)
    let program: Program = vec![(PrimName::Translate, Params::Translate(0, 0))];
    wm.add_hypothesis(program.clone());

    let prev = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let obs = prev.clone();

    wm.update(&prev, &obs);
    wm.update(&prev, &obs);
    wm.update(&prev, &obs);

    assert_eq!(wm.hypotheses[0].evidence_for, 3);
    assert_eq!(wm.hypotheses[0].status, HypothesisStatus::Confirmed);
}
