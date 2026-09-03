use std::collections::VecDeque;

use athlesia_arc_agi_3_adapter::bounded_episode_runtime::ArcAgi3BoundedEpisodeTermination;
use athlesia_arc_agi_3_adapter::competition_session_runtime::{
    ArcAgi3ScorecardId, ArcAgi3ScorecardRestProtocol, ArcAgi3ScorecardSummary,
};
use athlesia_arc_agi_3_adapter::ArcAgi3GameId;
use athlesia_arc_agi_3_blind_benchmark::execution_handoff::*;
use athlesia_arc_agi_3_blind_benchmark::execution_runtime::{
    ArcAgi3BlindBenchmarkExecutionError, ArcAgi3BlindBenchmarkExecutionRuntime,
    ArcAgi3BlindBenchmarkExecutionStatus,
};
use athlesia_arc_agi_3_blind_benchmark::harness_bridge::{
    ArcAgi3BlindBenchmarkExternalHarness, ArcAgi3BlindBenchmarkHarnessBridge,
    ArcAgi3BlindBenchmarkHarnessBridgeError, ArcAgi3BlindBenchmarkHarnessEpisode,
    ArcAgi3BlindBenchmarkHarnessFinalization, ArcAgi3BlindBenchmarkHarnessRequest,
    ArcAgi3BlindBenchmarkHarnessResponse,
};
use athlesia_arc_agi_3_blind_benchmark::run_binding::{
    ArcAgi3BlindBenchmarkBoundRun, ArcAgi3BlindBenchmarkRunBinding,
};
use athlesia_arc_agi_3_blind_benchmark::run_manifest::{
    ArcAgi3BlindBenchmarkBuildIdentity, ArcAgi3BlindBenchmarkConfigurationFingerprint,
    ArcAgi3BlindBenchmarkHarnessIdentity, ArcAgi3BlindBenchmarkProtocolIdentity,
    ArcAgi3BlindBenchmarkRunManifest,
};
use athlesia_arc_agi_3_blind_benchmark::{
    ArcAgi3BlindBenchmarkAgentIdentity, ArcAgi3BlindBenchmarkLedger, ArcAgi3BlindBenchmarkPolicy,
    ArcAgi3BlindBenchmarkRunId, ArcAgi3BlindBenchmarkSpec,
};
use serde_json::json;

const SOURCE_REVISION: &str = "08f748f8ba71669ce40b754abfc4c135e55b415b";

const CONFIG_FINGERPRINT: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn game_id(value: &str) -> ArcAgi3GameId {
    ArcAgi3GameId::new(value.to_string()).unwrap()
}

fn card_id(value: &str) -> ArcAgi3ScorecardId {
    ArcAgi3ScorecardId::new(value.to_string()).unwrap()
}

fn spec() -> ArcAgi3BlindBenchmarkSpec {
    ArcAgi3BlindBenchmarkSpec::new(
        ArcAgi3BlindBenchmarkRunId::new("handoff-run".to_string()).unwrap(),
        ArcAgi3BlindBenchmarkAgentIdentity::new(
            "athlesia".to_string(),
            "m53-handoff".to_string(),
            SOURCE_REVISION.to_string(),
        )
        .unwrap(),
        ArcAgi3BlindBenchmarkPolicy::new(8).unwrap(),
    )
}

fn manifest() -> ArcAgi3BlindBenchmarkRunManifest {
    ArcAgi3BlindBenchmarkRunManifest::new(
        spec(),
        ArcAgi3BlindBenchmarkHarnessIdentity::new("blind-harness".to_string(), "1.0".to_string())
            .unwrap(),
        ArcAgi3BlindBenchmarkBuildIdentity::new(
            SOURCE_REVISION.to_string(),
            "x86_64-unknown-linux-gnu".to_string(),
            "release".to_string(),
        )
        .unwrap(),
        ArcAgi3BlindBenchmarkProtocolIdentity::new(
            "arc-agi-3-blind-evaluation".to_string(),
            "2026-09".to_string(),
        )
        .unwrap(),
        ArcAgi3BlindBenchmarkConfigurationFingerprint::new(CONFIG_FINGERPRINT.to_string()).unwrap(),
        29,
    )
    .unwrap()
}

fn runtime() -> ArcAgi3BlindBenchmarkExecutionRuntime {
    ArcAgi3BlindBenchmarkExecutionRuntime::new(ArcAgi3BlindBenchmarkLedger::new(
        spec(),
        card_id("handoff-card"),
    ))
}

fn bound_run() -> ArcAgi3BlindBenchmarkBoundRun {
    ArcAgi3BlindBenchmarkRunBinding::bind(manifest(), runtime()).unwrap()
}

fn summary(score: f64) -> ArcAgi3ScorecardSummary {
    ArcAgi3ScorecardRestProtocol::decode_summary(json!({
        "card_id": "handoff-card",
        "score": score,
        "environments": [],
        "total_environments_completed": 1,
        "total_environments": 1,
        "total_levels_completed": 2,
        "total_levels": 2,
        "total_actions": 5,
        "competition_mode": true,
        "published_at":
            "2026-09-03T18:00:00Z",
        "authority": {
            "server": true
        }
    }))
    .unwrap()
}

fn episode_response(
    request_index: usize,
    request_budget: usize,
    game: &str,
    termination: ArcAgi3BoundedEpisodeTermination,
    completed_steps: usize,
) -> ArcAgi3BlindBenchmarkHarnessResponse {
    ArcAgi3BlindBenchmarkHarnessResponse::Episode(ArcAgi3BlindBenchmarkHarnessEpisode::new(
        request_index,
        request_budget,
        game_id(game),
        termination,
        completed_steps,
    ))
}

fn final_response(
    request_index: usize,
    request_budget: usize,
    score: f64,
) -> ArcAgi3BlindBenchmarkHarnessResponse {
    ArcAgi3BlindBenchmarkHarnessResponse::Finalized(ArcAgi3BlindBenchmarkHarnessFinalization::new(
        request_index,
        request_budget,
        summary(score),
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

    fn remaining(&self) -> usize {
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
            .expect("unexpected execution handoff request");

        assert_eq!(request.episode_index(), expected.0,);

        assert_eq!(request.max_cognitive_steps_per_episode(), expected.1,);

        self.responses
            .pop_front()
            .expect("missing execution handoff response")
    }
}

fn handoff(harness: ScriptedHarness) -> ArcAgi3BlindBenchmarkExecutionHandoff<ScriptedHarness> {
    ArcAgi3BlindBenchmarkExecutionHandoff::new(
        bound_run(),
        ArcAgi3BlindBenchmarkHarnessBridge::new(harness),
    )
}

#[test]
fn execution_handoff_accepts_prevalidated_bound_run_without_rebinding() {
    let source = include_str!("../src/execution_handoff.rs");

    assert!(source.contains("bound_run: ArcAgi3BlindBenchmarkBoundRun"));

    assert!(!source.contains("ArcAgi3BlindBenchmarkRunBinding::bind"));

    assert!(!source.contains("ArcAgi3BlindBenchmarkRunBinding::validate"));
}

#[test]
fn zero_episode_handoff_finalizes_exact_server_summary() {
    let harness = ScriptedHarness::new(vec![(0, 8)], vec![Ok(final_response(0, 8, 13.5))]);

    let mut handoff = handoff(harness);

    let result = handoff.execute().unwrap();

    assert_eq!(result.score(), 13.5,);

    assert_eq!(result.raw()["authority"]["server"], json!(true),);
}

#[test]
fn execution_handoff_preserves_manifest_identity_before_and_after_execution() {
    let harness = ScriptedHarness::new(vec![(0, 8)], vec![Ok(final_response(0, 8, 1.0))]);

    let mut handoff = handoff(harness);

    let run_id_before = handoff.manifest().spec().run_id().as_str().to_string();

    let source_before = handoff
        .manifest()
        .spec()
        .agent()
        .source_revision()
        .to_string();

    handoff.execute().unwrap();

    assert_eq!(handoff.manifest().spec().run_id().as_str(), run_id_before,);

    assert_eq!(
        handoff.manifest().spec().agent().source_revision(),
        source_before,
    );
}

#[test]
fn execution_handoff_records_harness_discovered_episode_then_finalizes() {
    let harness = ScriptedHarness::new(
        vec![(0, 8), (1, 8)],
        vec![
            Ok(episode_response(
                0,
                8,
                "unseen-game",
                ArcAgi3BoundedEpisodeTermination::Won,
                6,
            )),
            Ok(final_response(1, 8, 8.75)),
        ],
    );

    let mut handoff = handoff(harness);

    handoff.execute().unwrap();

    let (bound, _bridge) = handoff.into_parts();

    assert_eq!(bound.runtime().ledger().episodes().len(), 1,);

    assert_eq!(
        bound.runtime().ledger().episodes()[0].game_id(),
        &game_id("unseen-game"),
    );

    assert_eq!(bound.runtime().ledger().server_score(), Some(8.75,),);
}

#[test]
fn successful_handoff_finishes_runtime_in_finalized_state() {
    let harness = ScriptedHarness::new(vec![(0, 8)], vec![Ok(final_response(0, 8, 2.0))]);

    let mut handoff = handoff(harness);

    assert_eq!(
        handoff.execution_status(),
        ArcAgi3BlindBenchmarkExecutionStatus::Ready,
    );

    handoff.execute().unwrap();

    assert_eq!(
        handoff.execution_status(),
        ArcAgi3BlindBenchmarkExecutionStatus::Finalized,
    );
}

#[test]
fn external_harness_failure_faults_bound_runtime_without_fabrication() {
    let harness = ScriptedHarness::new(vec![(0, 8)], vec![Err("harness-offline")]);

    let mut handoff = handoff(harness);

    let result = handoff.execute();

    assert!(matches!(
        result,
        Err(ArcAgi3BlindBenchmarkExecutionError::Harness(
            ArcAgi3BlindBenchmarkHarnessBridgeError::Harness("harness-offline")
        ))
    ));

    assert_eq!(
        handoff.execution_status(),
        ArcAgi3BlindBenchmarkExecutionStatus::Faulted,
    );

    let (bound, _bridge) = handoff.into_parts();

    assert!(bound.runtime().ledger().episodes().is_empty());

    assert!(bound.runtime().ledger().final_summary().is_none());
}

#[test]
fn harness_request_identity_mismatch_faults_without_episode_commit() {
    let harness = ScriptedHarness::new(
        vec![(0, 8)],
        vec![Ok(episode_response(
            3,
            8,
            "wrong-index",
            ArcAgi3BoundedEpisodeTermination::Won,
            1,
        ))],
    );

    let mut handoff = handoff(harness);

    let result = handoff.execute();

    assert!(matches!(
        result,
        Err(ArcAgi3BlindBenchmarkExecutionError::Harness(
            ArcAgi3BlindBenchmarkHarnessBridgeError::EpisodeIndexMismatch {
                expected: 0,
                observed: 3,
            }
        ))
    ));

    assert_eq!(
        handoff.execution_status(),
        ArcAgi3BlindBenchmarkExecutionStatus::Faulted,
    );

    let (bound, _bridge) = handoff.into_parts();

    assert!(bound.runtime().ledger().episodes().is_empty());
}

#[test]
fn harness_budget_mismatch_faults_without_episode_commit() {
    let harness = ScriptedHarness::new(
        vec![(0, 8)],
        vec![Ok(episode_response(
            0,
            9,
            "wrong-budget",
            ArcAgi3BoundedEpisodeTermination::GameOver,
            1,
        ))],
    );

    let mut handoff = handoff(harness);

    let result = handoff.execute();

    assert!(matches!(
        result,
        Err(ArcAgi3BlindBenchmarkExecutionError::Harness(
            ArcAgi3BlindBenchmarkHarnessBridgeError::StepBudgetMismatch {
                expected: 8,
                observed: 9,
            }
        ))
    ));

    assert_eq!(
        handoff.execution_status(),
        ArcAgi3BlindBenchmarkExecutionStatus::Faulted,
    );
}

#[test]
fn faulted_execution_handoff_never_automatically_retries_harness() {
    let harness = ScriptedHarness::new(
        vec![(0, 8), (0, 8)],
        vec![Err("first-failure"), Ok(final_response(0, 8, 99.0))],
    );

    let mut handoff = handoff(harness);

    let first = handoff.execute();

    assert!(first.is_err());

    let second = handoff.execute();

    assert!(matches!(
        second,
        Err(ArcAgi3BlindBenchmarkExecutionError::RuntimeNotReady(
            ArcAgi3BlindBenchmarkExecutionStatus::Faulted,
        ))
    ));

    assert_eq!(handoff.bridge().harness().calls(), 1,);

    assert_eq!(handoff.bridge().harness().remaining(), 1,);
}

#[test]
fn finalized_execution_handoff_never_calls_harness_twice() {
    let harness = ScriptedHarness::new(
        vec![(0, 8), (0, 8)],
        vec![Ok(final_response(0, 8, 3.0)), Ok(final_response(0, 8, 4.0))],
    );

    let mut handoff = handoff(harness);

    handoff.execute().unwrap();

    let second = handoff.execute();

    assert!(matches!(
        second,
        Err(ArcAgi3BlindBenchmarkExecutionError::RuntimeNotReady(
            ArcAgi3BlindBenchmarkExecutionStatus::Finalized,
        ))
    ));

    assert_eq!(handoff.bridge().harness().calls(), 1,);

    assert_eq!(handoff.bridge().harness().remaining(), 1,);
}

#[test]
fn execution_handoff_preserves_exact_manifest_seed_and_configuration_fingerprint() {
    let harness = ScriptedHarness::new(vec![(0, 8)], vec![Ok(final_response(0, 8, 4.5))]);

    let mut handoff = handoff(harness);

    handoff.execute().unwrap();

    assert_eq!(handoff.manifest().deterministic_seed(), 29,);

    assert_eq!(
        handoff.manifest().configuration_fingerprint().as_str(),
        CONFIG_FINGERPRINT,
    );
}

#[test]
fn execution_handoff_preserves_exact_runtime_scorecard_identity() {
    let harness = ScriptedHarness::new(vec![(0, 8)], vec![Ok(final_response(0, 8, 6.0))]);

    let mut handoff = handoff(harness);

    handoff.execute().unwrap();

    let (bound, _bridge) = handoff.into_parts();

    assert_eq!(bound.runtime().ledger().card_id().as_str(), "handoff-card",);
}

#[test]
fn into_parts_preserves_exact_bound_run_and_harness_state() {
    let harness = ScriptedHarness::new(vec![(0, 8)], vec![Ok(final_response(0, 8, 7.0))]);

    let mut handoff = handoff(harness);

    handoff.execute().unwrap();

    let (bound, bridge) = handoff.into_parts();

    assert_eq!(bound.manifest().spec().run_id().as_str(), "handoff-run",);

    assert_eq!(bound.runtime().ledger().server_score(), Some(7.0,),);

    assert_eq!(bridge.harness().calls(), 1,);
}

#[test]
fn universal_facade_matches_direct_execution_handoff() {
    let direct_harness = ScriptedHarness::new(vec![(0, 8)], vec![Ok(final_response(0, 8, 12.0))]);

    let facade_harness = ScriptedHarness::new(vec![(0, 8)], vec![Ok(final_response(0, 8, 12.0))]);

    let mut direct = ArcAgi3BlindBenchmarkExecutionHandoff::new(
        bound_run(),
        ArcAgi3BlindBenchmarkHarnessBridge::new(direct_harness),
    );

    let mut facade = UniversalArcAgi3BlindBenchmarkExecutionHandoff::handoff(
        bound_run(),
        ArcAgi3BlindBenchmarkHarnessBridge::new(facade_harness),
    );

    let direct_score = direct.execute().unwrap().score();

    let facade_score = UniversalArcAgi3BlindBenchmarkExecutionHandoff::execute(&mut facade)
        .unwrap()
        .score();

    assert_eq!(direct_score, facade_score,);

    assert_eq!(direct.execution_status(), facade.execution_status(),);
}

#[test]
fn execution_handoff_contains_only_bound_run_to_harness_execution_authority() {
    let source = include_str!("../src/execution_handoff.rs");

    assert!(source.contains("ArcAgi3BlindBenchmarkBoundRun"));

    assert!(source.contains("run_blind_benchmark_with_harness"));

    assert!(source.contains("ArcAgi3BlindBenchmarkHarnessBridge"));

    assert!(!source.contains("run_with("));

    assert!(!source.contains("record_episode("));

    assert!(!source.contains(".finalize("));

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
            "M53 execution handoff leaked forbidden benchmark authority: {forbidden}",
        );
    }
}
