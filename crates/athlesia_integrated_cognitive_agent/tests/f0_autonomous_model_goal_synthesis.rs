use athlesia_integrated_cognitive_agent::{
    autonomous_cognitive_self_bootstrap::BootstrapObjectiveKind, AutonomousModelGoalSynthesis,
    AutonomousModelOutcomeEvidence,
};

use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

fn a(v: u64) -> CognitiveStructure {
    CognitiveStructure::atom(v)
}

fn s(v: u16) -> CognitiveSignal {
    CognitiveSignal::new(v).unwrap()
}

#[test]
fn strongest_empirically_controlled_prediction_becomes_endogenous_target() {
    let source = a(1);

    let action_a = a(10);
    let action_b = a(20);

    let outcome_a = a(100);
    let outcome_b = a(200);

    let result = AutonomousModelGoalSynthesis::synthesize(
        source,
        &[action_a.clone(), action_b.clone()],
        &[
            AutonomousModelOutcomeEvidence::new(action_a, outcome_a, s(900), s(600), s(100)),
            AutonomousModelOutcomeEvidence::new(
                action_b.clone(),
                outcome_b.clone(),
                s(800),
                s(950),
                s(100),
            ),
        ],
    )
    .expect("at least one empirically authorized prediction exists");

    assert_eq!(
        result.objective_kind(),
        BootstrapObjectiveKind::HypothesisDiscrimination,
    );

    assert_eq!(
        result.goal().identity(),
        &outcome_b,
        "zero information-gain tie must be resolved by stronger retained controllability",
    );

    assert_eq!(result.selected_seed_action(), &action_b,);

    assert_eq!(
        result.goal().priority(),
        s(800),
        "goal priority must be weakest-link confidence/controllability, not a fabricated constant",
    );
}

#[test]
fn unauthorized_prediction_cannot_seed_an_endogenous_goal() {
    let allowed = a(10);
    let forbidden = a(20);

    let result = AutonomousModelGoalSynthesis::synthesize(
        a(1),
        std::slice::from_ref(&allowed),
        &[
            AutonomousModelOutcomeEvidence::new(forbidden, a(999), s(1000), s(1000), s(1)),
            AutonomousModelOutcomeEvidence::new(allowed.clone(), a(111), s(700), s(700), s(100)),
        ],
    )
    .expect("authorized empirical prediction must survive");

    assert_eq!(result.selected_seed_action(), &allowed,);

    assert_eq!(result.goal().identity(), &a(111),);
}

#[test]
fn absent_empirical_predictions_do_not_fabricate_a_goal() {
    assert_eq!(
        AutonomousModelGoalSynthesis::synthesize(a(1), &[a(10), a(20)], &[],),
        None,
    );
}
