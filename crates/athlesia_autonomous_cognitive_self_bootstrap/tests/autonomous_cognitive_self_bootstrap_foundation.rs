use athlesia_autonomous_cognitive_self_bootstrap::*;
use athlesia_mindstone_sparse_cognition::CognitiveStructure;

fn a(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn s(value: u16) -> BootstrapSignal {
    BootstrapSignal::new(value).unwrap()
}

fn bounds() -> SelfBootstrapBounds {
    SelfBootstrapBounds::new(16, 32, 8).unwrap()
}

fn thresholds() -> SelfBootstrapThresholds {
    SelfBootstrapThresholds::new(s(500), s(500), s(500), s(500))
}

fn policy() -> SelfBootstrapPolicy {
    SelfBootstrapPolicy::new(bounds(), thresholds())
}

fn hypothesis(
    source: u64,
    action: u64,
    outcome: u64,
    confidence: u16,
    information: u16,
    controllability: u16,
    cost: u16,
) -> OutcomeHypothesis {
    OutcomeHypothesis::new(
        a(source),
        a(action),
        a(outcome),
        s(confidence),
        s(information),
        s(controllability),
        s(cost),
    )
}

fn input(
    affordances: Vec<CognitiveStructure>,
    hypotheses: Vec<OutcomeHypothesis>,
) -> SelfBootstrapInput {
    SelfBootstrapInput::new(
        a(1),
        None,
        affordances,
        hypotheses,
        BootstrapFeedback::Unspecified,
    )
}

#[test]
fn signal_and_frontier_bounds_are_exact() {
    assert_eq!(BootstrapSignal::new(0), Some(BootstrapSignal::zero()),);

    assert_eq!(BootstrapSignal::new(1000), Some(BootstrapSignal::maximum()),);

    assert_eq!(BootstrapSignal::new(1001), None,);
    assert_eq!(SelfBootstrapBounds::new(0, 1, 1), None,);

    assert_eq!(SelfBootstrapBounds::new(1, 0, 1), None,);

    assert_eq!(SelfBootstrapBounds::new(1, 1, 0), None,);
}

#[test]
fn empty_world_abstains_as_blocked() {
    let result = UniversalAutonomousCognitiveSelfBootstrap::evaluate(
        &input(Vec::new(), Vec::new()),
        policy(),
    )
    .unwrap();

    assert_eq!(result.status(), SelfBootstrapStatus::Blocked,);

    assert!(result.abstained());

    assert_eq!(
        result.objective().kind(),
        BootstrapObjectiveKind::ModelExpansion,
    );
}

#[test]
fn affordances_without_predictions_do_not_become_actions() {
    let result = UniversalAutonomousCognitiveSelfBootstrap::evaluate(
        &input(vec![a(10), a(20)], Vec::new()),
        policy(),
    )
    .unwrap();

    assert_eq!(result.status(), SelfBootstrapStatus::ModelExpansionRequired,);

    assert_eq!(result.selected(), None,);

    assert!(result.requires_model_expansion());
}

#[test]
fn unauthorized_hypothesis_is_rejected() {
    let result = UniversalAutonomousCognitiveSelfBootstrap::evaluate(
        &input(
            vec![a(10)],
            vec![hypothesis(1, 20, 200, 900, 900, 900, 100)],
        ),
        policy(),
    )
    .unwrap();

    assert_eq!(result.rejected_unauthorized_action_count(), 1,);

    assert_eq!(result.selected(), None,);
}

#[test]
fn source_state_mismatch_is_rejected() {
    let result = UniversalAutonomousCognitiveSelfBootstrap::evaluate(
        &input(
            vec![a(10)],
            vec![hypothesis(2, 10, 200, 900, 900, 900, 100)],
        ),
        policy(),
    )
    .unwrap();

    assert_eq!(result.rejected_source_state_count(), 1,);

    assert_eq!(result.selected(), None,);
}

#[test]
fn weak_confidence_is_rejected() {
    let result = UniversalAutonomousCognitiveSelfBootstrap::evaluate(
        &input(
            vec![a(10)],
            vec![hypothesis(1, 10, 200, 499, 900, 900, 100)],
        ),
        policy(),
    )
    .unwrap();

    assert_eq!(result.rejected_threshold_count(), 1,);
}

#[test]
fn weak_information_gain_is_rejected() {
    let result = UniversalAutonomousCognitiveSelfBootstrap::evaluate(
        &input(
            vec![a(10)],
            vec![hypothesis(1, 10, 200, 900, 499, 900, 100)],
        ),
        policy(),
    )
    .unwrap();

    assert_eq!(result.rejected_threshold_count(), 1,);
}

#[test]
fn weak_controllability_is_rejected() {
    let result = UniversalAutonomousCognitiveSelfBootstrap::evaluate(
        &input(
            vec![a(10)],
            vec![hypothesis(1, 10, 200, 900, 900, 499, 100)],
        ),
        policy(),
    )
    .unwrap();

    assert_eq!(result.rejected_threshold_count(), 1,);
}

#[test]
fn excessive_execution_cost_is_rejected() {
    let result = UniversalAutonomousCognitiveSelfBootstrap::evaluate(
        &input(
            vec![a(10)],
            vec![hypothesis(1, 10, 200, 900, 900, 900, 501)],
        ),
        policy(),
    )
    .unwrap();

    assert_eq!(result.rejected_threshold_count(), 1,);
}

#[test]
fn information_gain_is_primary_selection_authority() {
    let result = UniversalAutonomousCognitiveSelfBootstrap::evaluate(
        &input(
            vec![a(10), a(20)],
            vec![
                hypothesis(1, 10, 100, 900, 700, 900, 100),
                hypothesis(1, 20, 200, 700, 950, 600, 200),
            ],
        ),
        policy(),
    )
    .unwrap();

    assert_eq!(result.selected().unwrap().hypothesis().action(), &a(20),);
}

#[test]
fn controllability_breaks_information_ties() {
    let result = UniversalAutonomousCognitiveSelfBootstrap::evaluate(
        &input(
            vec![a(10), a(20)],
            vec![
                hypothesis(1, 10, 100, 900, 900, 700, 100),
                hypothesis(1, 20, 200, 700, 900, 950, 200),
            ],
        ),
        policy(),
    )
    .unwrap();

    assert_eq!(result.selected().unwrap().hypothesis().action(), &a(20),);
}

#[test]
fn lower_cost_breaks_equal_positive_evidence() {
    let result = UniversalAutonomousCognitiveSelfBootstrap::evaluate(
        &input(
            vec![a(10), a(20)],
            vec![
                hypothesis(1, 10, 100, 900, 900, 900, 300),
                hypothesis(1, 20, 200, 900, 900, 900, 100),
            ],
        ),
        policy(),
    )
    .unwrap();

    assert_eq!(result.selected().unwrap().hypothesis().action(), &a(20),);
}

#[test]
fn exact_ties_preserve_input_order() {
    let first = hypothesis(1, 10, 100, 900, 900, 900, 100);

    let second = hypothesis(1, 20, 200, 900, 900, 900, 100);

    let result = UniversalAutonomousCognitiveSelfBootstrap::evaluate(
        &input(vec![a(10), a(20)], vec![first, second]),
        policy(),
    )
    .unwrap();

    assert_eq!(result.selected().unwrap().input_index(), 0,);
}

#[test]
fn duplicate_exact_evidence_is_not_counted_twice() {
    let h = hypothesis(1, 10, 100, 900, 900, 900, 100);

    let result = UniversalAutonomousCognitiveSelfBootstrap::evaluate(
        &input(vec![a(10), a(10)], vec![h.clone(), h]),
        policy(),
    )
    .unwrap();

    assert_eq!(result.duplicate_affordance_count(), 1,);

    assert_eq!(result.duplicate_hypothesis_count(), 1,);

    assert_eq!(result.candidate_frontier().len(), 1,);
}

#[test]
fn terminal_success_never_dispatches_another_candidate() {
    let complete_input = SelfBootstrapInput::new(
        a(1),
        Some(a(0)),
        vec![a(10)],
        vec![hypothesis(1, 10, 200, 900, 900, 900, 100)],
        BootstrapFeedback::TerminalSuccess,
    );

    let result =
        UniversalAutonomousCognitiveSelfBootstrap::evaluate(&complete_input, policy()).unwrap();

    assert_eq!(result.status(), SelfBootstrapStatus::Complete,);

    assert_eq!(result.selected(), None,);

    assert_eq!(result.objective().target_state(), Some(&a(1)),);
}
