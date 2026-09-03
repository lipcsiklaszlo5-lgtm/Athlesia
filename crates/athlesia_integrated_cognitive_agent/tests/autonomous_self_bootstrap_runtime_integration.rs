use athlesia_integrated_cognitive_agent::{
    autonomous_cognitive_self_bootstrap::{
        BootstrapFeedback, BootstrapObjectiveKind, BootstrapSignal, OutcomeHypothesis,
        SelfBootstrapBounds, SelfBootstrapInput, SelfBootstrapPolicy, SelfBootstrapStatus,
        SelfBootstrapThresholds,
    },
    UniversalOnlineAutonomousCognitiveSelfBootstrap,
};
use athlesia_mindstone_sparse_cognition::CognitiveStructure;

fn a(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn s(value: u16) -> BootstrapSignal {
    BootstrapSignal::new(value).unwrap()
}

fn policy(frontier: usize) -> SelfBootstrapPolicy {
    SelfBootstrapPolicy::new(
        SelfBootstrapBounds::new(16, 32, frontier).unwrap(),
        SelfBootstrapThresholds::new(s(500), s(500), s(500), s(500)),
    )
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

fn evaluate(
    feedback: BootstrapFeedback,
    affordances: Vec<CognitiveStructure>,
    hypotheses: Vec<OutcomeHypothesis>,
    frontier: usize,
) -> athlesia_integrated_cognitive_agent::OnlineAutonomousSelfBootstrapDigest {
    UniversalOnlineAutonomousCognitiveSelfBootstrap::evaluate(
        &SelfBootstrapInput::new(a(1), Some(a(0)), affordances, hypotheses, feedback),
        policy(frontier),
    )
    .unwrap()
}

#[test]
fn m51_online_runtime_selects_exact_model_supported_action() {
    let digest = evaluate(
        BootstrapFeedback::Unspecified,
        vec![a(10), a(20)],
        vec![
            hypothesis(1, 10, 100, 900, 700, 900, 100),
            hypothesis(1, 20, 200, 900, 950, 900, 100),
        ],
        8,
    );

    assert_eq!(digest.status(), SelfBootstrapStatus::Selected,);

    assert_eq!(digest.selected_action(), Some(&a(20)),);

    assert_eq!(digest.predicted_outcome(), Some(&a(200)),);
}

#[test]
fn m51_online_runtime_preserves_exact_source_and_target_state() {
    let digest = evaluate(
        BootstrapFeedback::Unspecified,
        vec![a(10)],
        vec![hypothesis(1, 10, 100, 900, 900, 900, 100)],
        8,
    );

    assert_eq!(digest.source_state(), &a(1),);

    assert_eq!(digest.target_state(), Some(&a(100)),);
}

#[test]
fn m51_online_runtime_requires_model_expansion_without_predictions() {
    let digest = evaluate(BootstrapFeedback::Unspecified, vec![a(10)], Vec::new(), 8);

    assert_eq!(digest.status(), SelfBootstrapStatus::ModelExpansionRequired,);

    assert!(digest.requires_model_expansion());

    assert_eq!(digest.selected_action(), None,);
}

#[test]
fn m51_online_runtime_blocks_without_affordances() {
    let digest = evaluate(BootstrapFeedback::Unspecified, Vec::new(), Vec::new(), 8);

    assert_eq!(digest.status(), SelfBootstrapStatus::Blocked,);

    assert!(!digest.has_selected_action());
}

#[test]
fn m51_online_runtime_terminal_success_dispatches_nothing() {
    let digest = evaluate(
        BootstrapFeedback::TerminalSuccess,
        vec![a(10)],
        vec![hypothesis(1, 10, 100, 900, 900, 900, 100)],
        8,
    );

    assert!(digest.is_complete());

    assert_eq!(digest.selected_action(), None,);

    assert_eq!(digest.predicted_outcome(), None,);
}

#[test]
fn m51_online_runtime_rejects_unauthorized_actions() {
    let digest = evaluate(
        BootstrapFeedback::Unspecified,
        vec![a(10)],
        vec![hypothesis(1, 20, 200, 900, 900, 900, 100)],
        8,
    );

    assert_eq!(digest.rejected_unauthorized_action_count(), 1,);

    assert_eq!(digest.selected_action(), None,);
}

#[test]
fn m51_online_runtime_rejects_wrong_source_state() {
    let digest = evaluate(
        BootstrapFeedback::Unspecified,
        vec![a(10)],
        vec![hypothesis(2, 10, 100, 900, 900, 900, 100)],
        8,
    );

    assert_eq!(digest.rejected_source_state_count(), 1,);
}

#[test]
fn m51_online_runtime_preserves_threshold_rejection_authority() {
    let digest = evaluate(
        BootstrapFeedback::Unspecified,
        vec![a(10)],
        vec![hypothesis(1, 10, 100, 499, 900, 900, 100)],
        8,
    );

    assert_eq!(digest.rejected_threshold_count(), 1,);
}

#[test]
fn m51_online_runtime_preserves_duplicate_affordance_authority() {
    let digest = evaluate(
        BootstrapFeedback::Unspecified,
        vec![a(10), a(10)],
        vec![hypothesis(1, 10, 100, 900, 900, 900, 100)],
        8,
    );

    assert_eq!(digest.duplicate_affordance_count(), 1,);
}

#[test]
fn m51_online_runtime_preserves_duplicate_hypothesis_authority() {
    let value = hypothesis(1, 10, 100, 900, 900, 900, 100);

    let digest = evaluate(
        BootstrapFeedback::Unspecified,
        vec![a(10)],
        vec![value.clone(), value],
        8,
    );

    assert_eq!(digest.duplicate_hypothesis_count(), 1,);

    assert_eq!(digest.candidate_frontier_len(), 1,);
}

#[test]
fn m51_online_runtime_progress_synthesizes_progress_objective() {
    let digest = evaluate(
        BootstrapFeedback::Progress(s(800)),
        vec![a(10)],
        vec![hypothesis(1, 10, 100, 900, 900, 900, 100)],
        8,
    );

    assert_eq!(
        digest.objective_kind(),
        BootstrapObjectiveKind::ProgressContinuation,
    );
}

#[test]
fn m51_online_runtime_regression_synthesizes_recovery_objective() {
    let digest = evaluate(
        BootstrapFeedback::Regression(s(800)),
        vec![a(10)],
        vec![hypothesis(1, 10, 100, 900, 900, 900, 100)],
        8,
    );

    assert_eq!(
        digest.objective_kind(),
        BootstrapObjectiveKind::RecoveryExploration,
    );
}

#[test]
fn m51_online_runtime_terminal_failure_synthesizes_recovery_objective() {
    let digest = evaluate(
        BootstrapFeedback::TerminalFailure,
        vec![a(10)],
        vec![hypothesis(1, 10, 100, 900, 900, 900, 100)],
        8,
    );

    assert_eq!(
        digest.objective_kind(),
        BootstrapObjectiveKind::RecoveryExploration,
    );
}

#[test]
fn m51_online_runtime_preserves_frontier_truncation_authority() {
    let digest = evaluate(
        BootstrapFeedback::Unspecified,
        vec![a(10), a(20), a(30)],
        vec![
            hypothesis(1, 10, 100, 900, 700, 900, 100),
            hypothesis(1, 20, 200, 900, 800, 900, 100),
            hypothesis(1, 30, 300, 900, 900, 900, 100),
        ],
        2,
    );

    assert_eq!(digest.candidate_frontier_len(), 2,);

    assert!(digest.frontier_truncated());

    assert_eq!(digest.selected_action(), Some(&a(30)),);
}

#[test]
fn m51_online_runtime_is_deterministic_for_exact_input() {
    let input = SelfBootstrapInput::new(
        a(1),
        Some(a(0)),
        vec![a(10), a(20)],
        vec![
            hypothesis(1, 10, 100, 900, 900, 900, 100),
            hypothesis(1, 20, 200, 900, 900, 900, 100),
        ],
        BootstrapFeedback::Unspecified,
    );

    let first =
        UniversalOnlineAutonomousCognitiveSelfBootstrap::evaluate(&input, policy(8)).unwrap();

    let second =
        UniversalOnlineAutonomousCognitiveSelfBootstrap::evaluate(&input, policy(8)).unwrap();

    assert_eq!(first, second,);

    assert_eq!(first.selected_action(), Some(&a(10)),);
}
