use std::collections::VecDeque;

use athlesia_arc_agi_3_adapter::bounded_episode_runtime::ArcAgi3BoundedEpisodeTermination;
use athlesia_arc_agi_3_adapter::competition_session_runtime::{
    ArcAgi3ScorecardId, ArcAgi3ScorecardRestProtocol, ArcAgi3ScorecardSummary,
};
use athlesia_arc_agi_3_adapter::ArcAgi3GameId;
use athlesia_arc_agi_3_blind_benchmark::execution_runtime::{
    ArcAgi3BlindBenchmarkExecutionError, ArcAgi3BlindBenchmarkExecutionRuntime,
};
use athlesia_arc_agi_3_blind_benchmark::harness_bridge::*;
use athlesia_arc_agi_3_blind_benchmark::{
    ArcAgi3BlindBenchmarkAgentIdentity, ArcAgi3BlindBenchmarkLedger, ArcAgi3BlindBenchmarkPolicy,
    ArcAgi3BlindBenchmarkRunId, ArcAgi3BlindBenchmarkSpec,
};
use serde_json::json;

fn game_id(value: &str) -> ArcAgi3GameId {
    ArcAgi3GameId::new(value.to_string()).unwrap()
}

fn card_id(value: &str) -> ArcAgi3ScorecardId {
    ArcAgi3ScorecardId::new(value.to_string()).unwrap()
}

fn summary(card: &str, score: f64) -> ArcAgi3ScorecardSummary {
    ArcAgi3ScorecardRestProtocol::decode_summary(json!({
        "card_id": card,
        "score": score,
        "environments": [],
        "total_environments_completed": 2,
        "total_environments": 2,
        "total_levels_completed": 4,
        "total_levels": 4,
        "total_actions": 12,
        "competition_mode": true,
        "published_at":
            "2026-09-03T18:00:00Z",
        "server_authority": {
            "exact": true
        }
    }))
    .unwrap()
}

fn ledger() -> ArcAgi3BlindBenchmarkLedger {
    let run_id = ArcAgi3BlindBenchmarkRunId::new("blind-harness-run".to_string()).unwrap();

    let agent = ArcAgi3BlindBenchmarkAgentIdentity::new(
        "athlesia".to_string(),
        "m53-harness".to_string(),
        "f0d458d38f6783683874f99472809b4ad70dcbb4".to_string(),
    )
    .unwrap();

    let policy = ArcAgi3BlindBenchmarkPolicy::new(8).unwrap();

    ArcAgi3BlindBenchmarkLedger::new(
        ArcAgi3BlindBenchmarkSpec::new(run_id, agent, policy),
        card_id("blind-card"),
    )
}

fn runtime() -> ArcAgi3BlindBenchmarkExecutionRuntime {
    ArcAgi3BlindBenchmarkExecutionRuntime::new(ledger())
}

fn episode_response(
    request_episode_index: usize,
    request_budget: usize,
    game: &str,
    termination: ArcAgi3BoundedEpisodeTermination,
    completed_steps: usize,
) -> ArcAgi3BlindBenchmarkHarnessResponse {
    ArcAgi3BlindBenchmarkHarnessResponse::Episode(ArcAgi3BlindBenchmarkHarnessEpisode::new(
        request_episode_index,
        request_budget,
        game_id(game),
        termination,
        completed_steps,
    ))
}

fn final_response(
    request_episode_index: usize,
    request_budget: usize,
    score: f64,
) -> ArcAgi3BlindBenchmarkHarnessResponse {
    ArcAgi3BlindBenchmarkHarnessResponse::Finalized(ArcAgi3BlindBenchmarkHarnessFinalization::new(
        request_episode_index,
        request_budget,
        summary("blind-card", score),
    ))
}

struct ScriptedHarness {
    expected_requests: VecDeque<(usize, usize)>,
    responses: VecDeque<Result<ArcAgi3BlindBenchmarkHarnessResponse, &'static str>>,
    calls: usize,
}

impl ScriptedHarness {
    fn new(
        expected_requests: Vec<(usize, usize)>,
        responses: Vec<Result<ArcAgi3BlindBenchmarkHarnessResponse, &'static str>>,
    ) -> Self {
        Self {
            expected_requests: expected_requests.into(),
            responses: responses.into(),
            calls: 0,
        }
    }

    fn calls(&self) -> usize {
        self.calls
    }

    fn remaining_responses(&self) -> usize {
        self.responses.len()
    }
}

impl ArcAgi3BlindBenchmarkExternalHarness for ScriptedHarness {
    type Error = &'static str;

    fn next(
        &mut self,
        request: ArcAgi3BlindBenchmarkHarnessRequest,
    ) -> Result<ArcAgi3BlindBenchmarkHarnessResponse, Self::Error> {
        self.calls += 1;

        let expected = self
            .expected_requests
            .pop_front()
            .expect("unexpected harness request");

        assert_eq!(request.episode_index(), expected.0,);

        assert_eq!(request.max_cognitive_steps_per_episode(), expected.1,);

        self.responses
            .pop_front()
            .expect("missing scripted harness response")
    }
}

#[test]
fn bridge_request_preserves_only_episode_index_and_exact_budget() {
    let source = include_str!("../src/harness_bridge.rs");

    let start = source
        .find("pub struct ArcAgi3BlindBenchmarkHarnessRequest")
        .unwrap();

    let end = source
        .find("pub struct ArcAgi3BlindBenchmarkHarnessEpisode")
        .unwrap();

    let request_source = &source[start..end];

    assert!(request_source.contains("episode_index: usize"));

    assert!(request_source.contains("max_cognitive_steps_per_episode: usize"));

    assert!(!request_source.contains("game_id"));
}

#[test]
fn episode_response_maps_exact_observed_identity_termination_and_steps() {
    let harness = ScriptedHarness::new(
        vec![(0, 8), (1, 8)],
        vec![
            Ok(episode_response(
                0,
                8,
                "unknown-game",
                ArcAgi3BoundedEpisodeTermination::Won,
                5,
            )),
            Ok(final_response(1, 8, 3.0)),
        ],
    );

    let mut bridge = ArcAgi3BlindBenchmarkHarnessBridge::new(harness);

    let mut runtime = runtime();

    run_blind_benchmark_with_harness(&mut runtime, &mut bridge).unwrap();

    let observed = &runtime.ledger().episodes()[0];

    assert_eq!(observed.game_id(), &game_id("unknown-game"),);

    assert_eq!(
        observed.termination(),
        ArcAgi3BoundedEpisodeTermination::Won,
    );

    assert_eq!(observed.completed_cognitive_steps(), 5,);
}

#[test]
fn finalization_maps_exact_server_summary_without_recalculation() {
    let harness = ScriptedHarness::new(vec![(0, 8)], vec![Ok(final_response(0, 8, 37.125))]);

    let mut bridge = ArcAgi3BlindBenchmarkHarnessBridge::new(harness);

    let mut runtime = runtime();

    let result = run_blind_benchmark_with_harness(&mut runtime, &mut bridge).unwrap();

    assert_eq!(result.score(), 37.125,);

    assert_eq!(result.raw()["server_authority"]["exact"], json!(true),);
}

#[test]
fn episode_index_mismatch_is_rejected_before_execution_event_commit() {
    let harness = ScriptedHarness::new(
        vec![(0, 8)],
        vec![Ok(episode_response(
            99,
            8,
            "wrong-index",
            ArcAgi3BoundedEpisodeTermination::Won,
            1,
        ))],
    );

    let mut bridge = ArcAgi3BlindBenchmarkHarnessBridge::new(harness);

    let mut runtime = runtime();

    let result = run_blind_benchmark_with_harness(&mut runtime, &mut bridge);

    assert!(matches!(
        result,
        Err(ArcAgi3BlindBenchmarkExecutionError::Harness(
            ArcAgi3BlindBenchmarkHarnessBridgeError::EpisodeIndexMismatch {
                expected: 0,
                observed: 99,
            }
        ))
    ));

    assert!(runtime.ledger().episodes().is_empty());
}

#[test]
fn episode_budget_echo_mismatch_is_rejected_before_commit() {
    let harness = ScriptedHarness::new(
        vec![(0, 8)],
        vec![Ok(episode_response(
            0,
            9,
            "wrong-budget",
            ArcAgi3BoundedEpisodeTermination::Won,
            1,
        ))],
    );

    let mut bridge = ArcAgi3BlindBenchmarkHarnessBridge::new(harness);

    let mut runtime = runtime();

    let result = run_blind_benchmark_with_harness(&mut runtime, &mut bridge);

    assert!(matches!(
        result,
        Err(ArcAgi3BlindBenchmarkExecutionError::Harness(
            ArcAgi3BlindBenchmarkHarnessBridgeError::StepBudgetMismatch {
                expected: 8,
                observed: 9,
            }
        ))
    ));

    assert!(runtime.ledger().episodes().is_empty());
}

#[test]
fn finalization_index_mismatch_is_rejected_transactionally() {
    let harness = ScriptedHarness::new(vec![(0, 8)], vec![Ok(final_response(1, 8, 10.0))]);

    let mut bridge = ArcAgi3BlindBenchmarkHarnessBridge::new(harness);

    let mut runtime = runtime();

    let result = run_blind_benchmark_with_harness(&mut runtime, &mut bridge);

    assert!(matches!(
        result,
        Err(ArcAgi3BlindBenchmarkExecutionError::Harness(
            ArcAgi3BlindBenchmarkHarnessBridgeError::EpisodeIndexMismatch {
                expected: 0,
                observed: 1,
            }
        ))
    ));

    assert!(runtime.ledger().final_summary().is_none());
}

#[test]
fn finalization_budget_echo_mismatch_is_rejected_transactionally() {
    let harness = ScriptedHarness::new(vec![(0, 8)], vec![Ok(final_response(0, 7, 10.0))]);

    let mut bridge = ArcAgi3BlindBenchmarkHarnessBridge::new(harness);

    let mut runtime = runtime();

    let result = run_blind_benchmark_with_harness(&mut runtime, &mut bridge);

    assert!(matches!(
        result,
        Err(ArcAgi3BlindBenchmarkExecutionError::Harness(
            ArcAgi3BlindBenchmarkHarnessBridgeError::StepBudgetMismatch {
                expected: 8,
                observed: 7,
            }
        ))
    ));

    assert!(runtime.ledger().final_summary().is_none());
}

#[test]
fn external_harness_failure_is_preserved_as_execution_harness_failure() {
    let harness = ScriptedHarness::new(vec![(0, 8)], vec![Err("external-harness-failed")]);

    let mut bridge = ArcAgi3BlindBenchmarkHarnessBridge::new(harness);

    let mut runtime = runtime();

    let result = run_blind_benchmark_with_harness(&mut runtime, &mut bridge);

    assert!(matches!(
        result,
        Err(ArcAgi3BlindBenchmarkExecutionError::Harness(
            ArcAgi3BlindBenchmarkHarnessBridgeError::Harness("external-harness-failed")
        ))
    ));
}

#[test]
fn bridge_invokes_external_harness_once_per_execution_request() {
    let harness = ScriptedHarness::new(
        vec![(0, 8), (1, 8), (2, 8)],
        vec![
            Ok(episode_response(
                0,
                8,
                "a",
                ArcAgi3BoundedEpisodeTermination::GameOver,
                2,
            )),
            Ok(episode_response(
                1,
                8,
                "b",
                ArcAgi3BoundedEpisodeTermination::Won,
                3,
            )),
            Ok(final_response(2, 8, 5.0)),
        ],
    );

    let mut bridge = ArcAgi3BlindBenchmarkHarnessBridge::new(harness);

    let mut runtime = runtime();

    run_blind_benchmark_with_harness(&mut runtime, &mut bridge).unwrap();

    assert_eq!(bridge.harness().calls(), 3,);
}

#[test]
fn integrated_bridge_preserves_harness_discovered_episode_order() {
    let harness = ScriptedHarness::new(
        vec![(0, 8), (1, 8), (2, 8)],
        vec![
            Ok(episode_response(
                0,
                8,
                "first-hidden",
                ArcAgi3BoundedEpisodeTermination::GameOver,
                4,
            )),
            Ok(episode_response(
                1,
                8,
                "second-hidden",
                ArcAgi3BoundedEpisodeTermination::Won,
                6,
            )),
            Ok(final_response(2, 8, 11.0)),
        ],
    );

    let mut bridge = ArcAgi3BlindBenchmarkHarnessBridge::new(harness);

    let mut runtime = runtime();

    run_blind_benchmark_with_harness(&mut runtime, &mut bridge).unwrap();

    let episodes = runtime.ledger().episodes();

    assert_eq!(episodes.len(), 2,);

    assert_eq!(episodes[0].game_id(), &game_id("first-hidden"),);

    assert_eq!(episodes[1].game_id(), &game_id("second-hidden"),);

    assert_eq!(runtime.ledger().server_score(), Some(11.0),);
}

#[test]
fn request_mismatch_faults_execution_runtime_and_prevents_automatic_retry() {
    let harness = ScriptedHarness::new(
        vec![(0, 8), (0, 8)],
        vec![
            Ok(episode_response(
                4,
                8,
                "bad",
                ArcAgi3BoundedEpisodeTermination::Won,
                1,
            )),
            Ok(final_response(0, 8, 99.0)),
        ],
    );

    let mut bridge = ArcAgi3BlindBenchmarkHarnessBridge::new(harness);

    let mut runtime = runtime();

    let _ = run_blind_benchmark_with_harness(&mut runtime, &mut bridge);

    let second = run_blind_benchmark_with_harness(&mut runtime, &mut bridge);

    assert!(matches!(
        second,
        Err(ArcAgi3BlindBenchmarkExecutionError::RuntimeNotReady(_))
    ));

    assert_eq!(bridge.harness().calls(), 1,);

    assert_eq!(bridge.harness().remaining_responses(), 1,);
}

#[test]
fn bridge_never_mutates_foundation_ledger_directly() {
    let source = include_str!("../src/harness_bridge.rs");

    assert!(!source.contains(".record_episode("));

    assert!(!source.contains(".finalize("));

    assert!(source.contains("runtime.run_with"));
}

#[test]
fn into_harness_preserves_exact_external_harness_state() {
    let harness = ScriptedHarness::new(vec![(0, 8)], vec![Ok(final_response(0, 8, 1.0))]);

    let bridge = ArcAgi3BlindBenchmarkHarnessBridge::new(harness);

    let recovered = bridge.into_harness();

    assert_eq!(recovered.calls(), 0,);

    assert_eq!(recovered.remaining_responses(), 1,);
}

#[test]
fn universal_facade_matches_direct_harness_bridge_execution() {
    let harness_a = ScriptedHarness::new(vec![(0, 8)], vec![Ok(final_response(0, 8, 4.0))]);

    let harness_b = ScriptedHarness::new(vec![(0, 8)], vec![Ok(final_response(0, 8, 4.0))]);

    let mut direct_bridge = ArcAgi3BlindBenchmarkHarnessBridge::new(harness_a);

    let mut facade_bridge = UniversalArcAgi3BlindBenchmarkHarnessBridge::bridge(harness_b);

    let mut direct_runtime = runtime();

    let mut facade_runtime = runtime();

    run_blind_benchmark_with_harness(&mut direct_runtime, &mut direct_bridge).unwrap();

    UniversalArcAgi3BlindBenchmarkHarnessBridge::run(&mut facade_runtime, &mut facade_bridge)
        .unwrap();

    assert_eq!(
        direct_runtime.ledger().server_score(),
        facade_runtime.ledger().server_score(),
    );

    assert_eq!(
        direct_bridge.harness().calls(),
        facade_bridge.harness().calls(),
    );
}

#[test]
fn harness_bridge_contains_no_hidden_catalog_transport_action_score_or_session_authority() {
    let source = include_str!("../src/harness_bridge.rs");

    for forbidden in [
        "game_catalog",
        "environment_catalog",
        "hidden_games",
        "hidden_environments",
        "evaluation_games",
        "public_games",
        "RHAE",
        "baseline_actions",
        ".powi(",
        "Action1",
        "Action2",
        "Action3",
        "Action4",
        "Action5",
        "Action6",
        "Action7",
        "reqwest",
        "ureq",
        "TcpStream",
        "UdpSocket",
        "ArcAgi3CompetitionSession",
        "ArcAgi3RestTransport",
        "open_scorecard",
        "close_scorecard",
        "begin_reset",
        ".reset(",
        "retry(",
        "sleep(",
    ] {
        assert!(
            !source.contains(forbidden,),
            "M53 harness bridge leaked forbidden benchmark authority: {forbidden}",
        );
    }

    assert!(source.contains("ArcAgi3BlindBenchmarkExternalHarness"));

    assert!(source.contains("EpisodeIndexMismatch"));

    assert!(source.contains("StepBudgetMismatch"));
}
