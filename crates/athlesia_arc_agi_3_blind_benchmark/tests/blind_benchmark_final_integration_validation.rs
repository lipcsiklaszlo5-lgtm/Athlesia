use std::collections::VecDeque;

use athlesia_arc_agi_3_adapter::bounded_episode_runtime::ArcAgi3BoundedEpisodeTermination;
use athlesia_arc_agi_3_adapter::competition_session_runtime::{
    ArcAgi3ScorecardId, ArcAgi3ScorecardRestProtocol, ArcAgi3ScorecardSummary,
};
use athlesia_arc_agi_3_adapter::ArcAgi3GameId;
use athlesia_arc_agi_3_blind_benchmark::execution_handoff::{
    ArcAgi3BlindBenchmarkExecutionHandoff, UniversalArcAgi3BlindBenchmarkExecutionHandoff,
};
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
use athlesia_arc_agi_3_blind_benchmark::result_record::{
    ArcAgi3BlindBenchmarkResultRecord, ArcAgi3BlindBenchmarkResultRecordError,
    UniversalArcAgi3BlindBenchmarkResultRecord,
};
use athlesia_arc_agi_3_blind_benchmark::run_binding::{
    ArcAgi3BlindBenchmarkBoundRun, ArcAgi3BlindBenchmarkRunBinding,
    UniversalArcAgi3BlindBenchmarkRunBinding,
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

const SOURCE_REVISION: &str = "35795484ad5b2f0e64d87914e8efdc39294cbefa";

const CONFIG_FINGERPRINT: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

fn game_id(value: &str) -> ArcAgi3GameId {
    ArcAgi3GameId::new(value.to_string()).unwrap()
}

fn card_id() -> ArcAgi3ScorecardId {
    ArcAgi3ScorecardId::new("final-validation-card".to_string()).unwrap()
}

fn spec() -> ArcAgi3BlindBenchmarkSpec {
    ArcAgi3BlindBenchmarkSpec::new(
        ArcAgi3BlindBenchmarkRunId::new("final-validation-run".to_string()).unwrap(),
        ArcAgi3BlindBenchmarkAgentIdentity::new(
            "athlesia".to_string(),
            "m53-final-validation".to_string(),
            SOURCE_REVISION.to_string(),
        )
        .unwrap(),
        ArcAgi3BlindBenchmarkPolicy::new(11).unwrap(),
    )
}

fn run_manifest() -> ArcAgi3BlindBenchmarkRunManifest {
    ArcAgi3BlindBenchmarkRunManifest::new(
        spec(),
        ArcAgi3BlindBenchmarkHarnessIdentity::new(
            "official-blind-harness".to_string(),
            "1.0".to_string(),
        )
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
        101,
    )
    .unwrap()
}

fn runtime() -> ArcAgi3BlindBenchmarkExecutionRuntime {
    ArcAgi3BlindBenchmarkExecutionRuntime::new(ArcAgi3BlindBenchmarkLedger::new(spec(), card_id()))
}

fn bound_run() -> ArcAgi3BlindBenchmarkBoundRun {
    ArcAgi3BlindBenchmarkRunBinding::bind(run_manifest(), runtime()).unwrap()
}

fn summary(score: f64) -> ArcAgi3ScorecardSummary {
    ArcAgi3ScorecardRestProtocol::decode_summary(json!({
        "card_id":
            "final-validation-card",
        "score": score,
        "environments": [],
        "total_environments_completed": 2,
        "total_environments": 2,
        "total_levels_completed": 4,
        "total_levels": 4,
        "total_actions": 12,
        "competition_mode": true,
        "published_at":
            "2026-09-03T18:45:00Z",
        "future_server_evidence": {
            "preserve": true,
            "revision": 7
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
            .expect("unexpected final-validation harness request");

        assert_eq!(request.episode_index(), expected.0,);

        assert_eq!(request.max_cognitive_steps_per_episode(), expected.1,);

        self.responses
            .pop_front()
            .expect("missing final-validation harness response")
    }
}

fn successful_handoff(score: f64) -> ArcAgi3BlindBenchmarkExecutionHandoff<ScriptedHarness> {
    ArcAgi3BlindBenchmarkExecutionHandoff::new(
        bound_run(),
        ArcAgi3BlindBenchmarkHarnessBridge::new(ScriptedHarness::new(
            vec![(0, 11), (1, 11), (2, 11)],
            vec![
                Ok(episode_response(
                    0,
                    11,
                    "blind-unseen-alpha",
                    ArcAgi3BoundedEpisodeTermination::Won,
                    7,
                )),
                Ok(episode_response(
                    1,
                    11,
                    "blind-unseen-beta",
                    ArcAgi3BoundedEpisodeTermination::GameOver,
                    11,
                )),
                Ok(final_response(2, 11, score)),
            ],
        )),
    )
}

fn successful_record(score: f64) -> ArcAgi3BlindBenchmarkResultRecord {
    let mut handoff = successful_handoff(score);

    handoff.execute().unwrap();

    let (bound, bridge) = handoff.into_parts();

    assert_eq!(bridge.harness().calls(), 3,);

    ArcAgi3BlindBenchmarkResultRecord::from_bound_run(bound).unwrap()
}

#[test]
fn final_chain_runs_spec_manifest_ledger_binding_handoff_and_result_record() {
    let record = successful_record(21.5);

    assert_eq!(
        record.manifest().spec().run_id().as_str(),
        "final-validation-run",
    );

    assert_eq!(record.episodes().len(), 2,);

    assert_eq!(record.server_score(), Some(21.5,),);
}

#[test]
fn final_chain_preserves_exact_agent_identity_and_source_revision() {
    let record = successful_record(1.0);

    let agent = record.manifest().spec().agent();

    assert_eq!(agent.name(), "athlesia",);

    assert_eq!(agent.version(), "m53-final-validation",);

    assert_eq!(agent.source_revision(), SOURCE_REVISION,);
}

#[test]
fn final_chain_preserves_exact_harness_build_and_protocol_identity() {
    let record = successful_record(2.0);

    assert_eq!(
        record.manifest().harness_identity().name(),
        "official-blind-harness",
    );

    assert_eq!(record.manifest().harness_identity().version(), "1.0",);

    assert_eq!(
        record.manifest().build_identity().source_revision(),
        SOURCE_REVISION,
    );

    assert_eq!(
        record.manifest().protocol_identity().name(),
        "arc-agi-3-blind-evaluation",
    );

    assert_eq!(record.manifest().protocol_identity().revision(), "2026-09",);
}

#[test]
fn final_chain_preserves_configuration_fingerprint_seed_and_step_budget() {
    let record = successful_record(3.0);

    assert_eq!(
        record.manifest().configuration_fingerprint().as_str(),
        CONFIG_FINGERPRINT,
    );

    assert_eq!(record.manifest().deterministic_seed(), 101,);

    assert_eq!(
        record
            .manifest()
            .spec()
            .policy()
            .max_cognitive_steps_per_episode(),
        11,
    );
}

#[test]
fn final_chain_preserves_exact_scorecard_identity() {
    let record = successful_record(4.0);

    assert_eq!(record.scorecard_id().as_str(), "final-validation-card",);

    assert_eq!(
        record.final_summary().unwrap().card_id().as_str(),
        "final-validation-card",
    );
}

#[test]
fn final_chain_preserves_harness_discovered_episode_order_only_after_execution() {
    let bound = bound_run();

    assert!(bound.runtime().ledger().episodes().is_empty());

    let harness = ScriptedHarness::new(
        vec![(0, 11), (1, 11), (2, 11)],
        vec![
            Ok(episode_response(
                0,
                11,
                "blind-unseen-alpha",
                ArcAgi3BoundedEpisodeTermination::Won,
                7,
            )),
            Ok(episode_response(
                1,
                11,
                "blind-unseen-beta",
                ArcAgi3BoundedEpisodeTermination::GameOver,
                11,
            )),
            Ok(final_response(2, 11, 5.0)),
        ],
    );

    let mut handoff = ArcAgi3BlindBenchmarkExecutionHandoff::new(
        bound,
        ArcAgi3BlindBenchmarkHarnessBridge::new(harness),
    );

    handoff.execute().unwrap();

    let (bound, _bridge) = handoff.into_parts();

    assert_eq!(
        bound.runtime().ledger().episodes()[0].game_id(),
        &game_id("blind-unseen-alpha",),
    );

    assert_eq!(
        bound.runtime().ledger().episodes()[1].game_id(),
        &game_id("blind-unseen-beta",),
    );
}

#[test]
fn final_chain_preserves_exact_episode_termination_and_step_provenance() {
    let record = successful_record(6.0);

    assert_eq!(
        record.episodes()[0].termination(),
        ArcAgi3BoundedEpisodeTermination::Won,
    );

    assert_eq!(record.episodes()[0].completed_cognitive_steps(), 7,);

    assert_eq!(
        record.episodes()[1].termination(),
        ArcAgi3BoundedEpisodeTermination::GameOver,
    );

    assert_eq!(record.episodes()[1].completed_cognitive_steps(), 11,);
}

#[test]
fn final_chain_preserves_exact_server_score_without_recalculation() {
    let record = successful_record(37.125);

    assert_eq!(record.server_score(), Some(37.125,),);

    assert_eq!(record.final_summary().unwrap().score(), 37.125,);
}

#[test]
fn final_chain_preserves_unknown_future_server_summary_fields() {
    let record = successful_record(8.0);

    let raw = record.final_summary().unwrap().raw();

    assert_eq!(raw["future_server_evidence"]["preserve"], json!(true),);

    assert_eq!(raw["future_server_evidence"]["revision"], json!(7),);
}

#[test]
fn final_chain_requests_are_derived_only_from_history_and_frozen_budget() {
    let harness = ScriptedHarness::new(
        vec![(0, 11), (1, 11)],
        vec![
            Ok(episode_response(
                0,
                11,
                "discovered-one",
                ArcAgi3BoundedEpisodeTermination::Won,
                3,
            )),
            Ok(final_response(1, 11, 9.0)),
        ],
    );

    let mut handoff = ArcAgi3BlindBenchmarkExecutionHandoff::new(
        bound_run(),
        ArcAgi3BlindBenchmarkHarnessBridge::new(harness),
    );

    handoff.execute().unwrap();

    assert_eq!(handoff.bridge().harness().calls(), 2,);
}

#[test]
fn final_chain_harness_failure_faults_without_fabricated_result_record() {
    let harness = ScriptedHarness::new(vec![(0, 11)], vec![Err("blind-harness-failure")]);

    let mut handoff = ArcAgi3BlindBenchmarkExecutionHandoff::new(
        bound_run(),
        ArcAgi3BlindBenchmarkHarnessBridge::new(harness),
    );

    let result = handoff.execute();

    assert!(matches!(
        result,
        Err(ArcAgi3BlindBenchmarkExecutionError::Harness(
            ArcAgi3BlindBenchmarkHarnessBridgeError::Harness("blind-harness-failure")
        ))
    ));

    let (bound, _bridge) = handoff.into_parts();

    assert_eq!(
        bound.runtime().status(),
        ArcAgi3BlindBenchmarkExecutionStatus::Faulted,
    );

    assert!(matches!(
        ArcAgi3BlindBenchmarkResultRecord::from_bound_run(bound,),
        Err(ArcAgi3BlindBenchmarkResultRecordError::RuntimeNotFinalized(
            ArcAgi3BlindBenchmarkExecutionStatus::Faulted,
        ))
    ));
}

#[test]
fn final_chain_identity_mismatch_faults_before_episode_commit() {
    let harness = ScriptedHarness::new(
        vec![(0, 11)],
        vec![Ok(episode_response(
            4,
            11,
            "invalid-index",
            ArcAgi3BoundedEpisodeTermination::Won,
            2,
        ))],
    );

    let mut handoff = ArcAgi3BlindBenchmarkExecutionHandoff::new(
        bound_run(),
        ArcAgi3BlindBenchmarkHarnessBridge::new(harness),
    );

    let result = handoff.execute();

    assert!(matches!(
        result,
        Err(ArcAgi3BlindBenchmarkExecutionError::Harness(
            ArcAgi3BlindBenchmarkHarnessBridgeError::EpisodeIndexMismatch {
                expected: 0,
                observed: 4,
            }
        ))
    ));

    let (bound, _bridge) = handoff.into_parts();

    assert!(bound.runtime().ledger().episodes().is_empty());
}

#[test]
fn final_chain_never_automatically_retries_after_failure() {
    let harness = ScriptedHarness::new(
        vec![(0, 11), (0, 11)],
        vec![Err("one-shot-failure"), Ok(final_response(0, 11, 999.0))],
    );

    let mut handoff = ArcAgi3BlindBenchmarkExecutionHandoff::new(
        bound_run(),
        ArcAgi3BlindBenchmarkHarnessBridge::new(harness),
    );

    assert!(handoff.execute().is_err());

    assert!(matches!(
        handoff.execute(),
        Err(ArcAgi3BlindBenchmarkExecutionError::RuntimeNotReady(
            ArcAgi3BlindBenchmarkExecutionStatus::Faulted,
        ))
    ));

    assert_eq!(handoff.bridge().harness().calls(), 1,);

    assert_eq!(handoff.bridge().harness().remaining(), 1,);
}

#[test]
fn universal_facades_preserve_full_chain_equivalence() {
    let manifest = run_manifest();

    let runtime = runtime();

    let bound = UniversalArcAgi3BlindBenchmarkRunBinding::bind(manifest, runtime).unwrap();

    let harness = ScriptedHarness::new(vec![(0, 11)], vec![Ok(final_response(0, 11, 14.0))]);

    let mut handoff = UniversalArcAgi3BlindBenchmarkExecutionHandoff::handoff(
        bound,
        ArcAgi3BlindBenchmarkHarnessBridge::new(harness),
    );

    UniversalArcAgi3BlindBenchmarkExecutionHandoff::execute(&mut handoff).unwrap();

    let (bound, _bridge) = handoff.into_parts();

    let record = UniversalArcAgi3BlindBenchmarkResultRecord::record(bound).unwrap();

    assert_eq!(
        record.manifest().spec().run_id().as_str(),
        "final-validation-run",
    );

    assert_eq!(record.server_score(), Some(14.0,),);
}

#[test]
fn final_integration_layer_adds_no_benchmark_intelligence_or_transport_authority() {
    let source = include_str!("blind_benchmark_final_integration_validation.rs");

    let forbidden = [
        ["Action", "1"].concat(),
        ["Action", "2"].concat(),
        ["Action", "3"].concat(),
        ["Action", "4"].concat(),
        ["Action", "5"].concat(),
        ["Action", "6"].concat(),
        ["Action", "7"].concat(),
        ["RH", "AE"].concat(),
        ["baseline", "_actions"].concat(),
        [".", "powi("].concat(),
        ["req", "west"].concat(),
        ["u", "req"].concat(),
        ["Tcp", "Stream"].concat(),
        ["Udp", "Socket"].concat(),
        ["ArcAgi3Rest", "Transport"].concat(),
        ["open", "_scorecard"].concat(),
        ["close", "_scorecard"].concat(),
        ["begin", "_reset"].concat(),
        [".", "reset("].concat(),
        ["retry", "("].concat(),
        ["sleep", "("].concat(),
        ["game", "_catalog"].concat(),
        ["environment", "_catalog"].concat(),
        ["hidden", "_games"].concat(),
        ["hidden", "_environments"].concat(),
        ["evaluation", "_games"].concat(),
        ["public", "_games"].concat(),
    ];

    for token in forbidden {
        assert!(
            !source.contains(&token),
            "M53 final validation leaked forbidden authority: {token}",
        );
    }
}
