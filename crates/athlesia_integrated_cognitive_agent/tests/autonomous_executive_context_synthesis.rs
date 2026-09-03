use athlesia_executive_agency::{ColdStartExplorationPolicy, ColdStartExplorationStatus};
use athlesia_integrated_cognitive_agent::{
    autonomous_cognitive_self_bootstrap::{
        BootstrapFeedback, BootstrapSignal, OutcomeHypothesis, SelfBootstrapBounds,
        SelfBootstrapInput, SelfBootstrapPolicy, SelfBootstrapThresholds,
    },
    OnlineAutonomousColdStartExplorationError, OnlineAutonomousColdStartExplorationPolicy,
    UniversalOnlineAutonomousCognitiveSelfBootstrap,
    UniversalOnlineAutonomousColdStartExplorationSynthesis,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn cognitive(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn bootstrap(value: u16) -> BootstrapSignal {
    BootstrapSignal::new(value).unwrap()
}

fn bootstrap_policy() -> SelfBootstrapPolicy {
    SelfBootstrapPolicy::new(
        SelfBootstrapBounds::new(16, 32, 8).unwrap(),
        SelfBootstrapThresholds::new(
            bootstrap(500),
            bootstrap(500),
            bootstrap(500),
            bootstrap(500),
        ),
    )
}

fn selected_digest() -> athlesia_integrated_cognitive_agent::OnlineAutonomousSelfBootstrapDigest {
    UniversalOnlineAutonomousCognitiveSelfBootstrap::evaluate(
        &SelfBootstrapInput::new(
            atom(1),
            Some(atom(0)),
            vec![atom(10)],
            vec![OutcomeHypothesis::new(
                atom(1),
                atom(10),
                atom(100),
                bootstrap(900),
                bootstrap(800),
                bootstrap(700),
                bootstrap(100),
            )],
            BootstrapFeedback::Unspecified,
        ),
        bootstrap_policy(),
    )
    .unwrap()
}

fn model_expansion_digest(
) -> athlesia_integrated_cognitive_agent::OnlineAutonomousSelfBootstrapDigest {
    UniversalOnlineAutonomousCognitiveSelfBootstrap::evaluate(
        &SelfBootstrapInput::new(
            atom(1),
            Some(atom(0)),
            vec![atom(10)],
            Vec::new(),
            BootstrapFeedback::Unspecified,
        ),
        bootstrap_policy(),
    )
    .unwrap()
}

fn cold_start_policy() -> ColdStartExplorationPolicy {
    ColdStartExplorationPolicy::new(
        16,
        16,
        cognitive(500),
        cognitive(500),
        cognitive(500),
        cognitive(1),
    )
    .unwrap()
}

fn synthesis_policy() -> OnlineAutonomousColdStartExplorationPolicy {
    OnlineAutonomousColdStartExplorationPolicy::new(cognitive(1000), cold_start_policy())
}

fn bundle() -> athlesia_integrated_cognitive_agent::OnlineAutonomousColdStartExplorationBundle {
    UniversalOnlineAutonomousColdStartExplorationSynthesis::evaluate(
        &selected_digest(),
        synthesis_policy(),
    )
    .unwrap()
}

#[test]
fn selected_bootstrap_digest_builds_cold_start_exploration_bundle() {
    let value = bundle();

    assert_eq!(
        value.result().status(),
        ColdStartExplorationStatus::Selected,
    );
}

#[test]
fn autonomous_goal_identity_is_exact_bootstrap_target() {
    assert_eq!(bundle().goal().identity(), &atom(100),);
}

#[test]
fn autonomous_goal_priority_is_domain_general_policy_authority() {
    assert_eq!(bundle().goal().priority(), cognitive(1000),);
}

#[test]
fn autonomous_cold_start_goal_begins_unsatisfied() {
    assert_eq!(bundle().goal().satisfaction(), cognitive(0),);
}

#[test]
fn cold_start_candidate_preserves_exact_selected_action() {
    assert_eq!(bundle().candidate().action(), &atom(10),);
}

#[test]
fn cold_start_candidate_preserves_exact_predicted_outcome() {
    assert_eq!(bundle().candidate().predicted_outcome(), &atom(100),);
}

#[test]
fn cold_start_candidate_preserves_exact_bootstrap_evidence() {
    let value = bundle();
    let signals = value.candidate().signals();

    assert_eq!(signals.expected_information_gain(), cognitive(800),);

    assert_eq!(signals.controllability(), cognitive(700),);

    assert_eq!(signals.evidence_confidence(), cognitive(900),);

    assert_eq!(signals.execution_cost(), cognitive(100),);
}

#[test]
fn cold_start_explicitly_has_no_fabricated_learning_progress() {
    assert_eq!(
        bundle().candidate().signals().learning_progress(),
        cognitive(0),
    );
}

#[test]
fn cold_start_selection_preserves_exact_action_and_prediction() {
    let value = bundle();

    let selected = value.selected_exploration().candidate();

    assert_eq!(selected.action(), &atom(10),);

    assert_eq!(selected.predicted_outcome(), &atom(100),);
}

#[test]
fn cold_start_value_uses_grounded_bottleneck_minus_execution_cost() {
    assert_eq!(
        bundle().selected_exploration().net_exploration_value(),
        cognitive(600),
    );
}

#[test]
fn model_expansion_state_cannot_manufacture_exploration() {
    let error = UniversalOnlineAutonomousColdStartExplorationSynthesis::evaluate(
        &model_expansion_digest(),
        synthesis_policy(),
    )
    .err()
    .unwrap();

    assert_eq!(
        error,
        OnlineAutonomousColdStartExplorationError::BootstrapDecisionNotSelected,
    );
}

#[test]
fn cold_start_synthesis_is_deterministic_for_exact_input() {
    let digest = selected_digest();

    let first = UniversalOnlineAutonomousColdStartExplorationSynthesis::evaluate(
        &digest,
        synthesis_policy(),
    )
    .unwrap();

    let second = UniversalOnlineAutonomousColdStartExplorationSynthesis::evaluate(
        &digest,
        synthesis_policy(),
    )
    .unwrap();

    assert_eq!(first, second,);
}
