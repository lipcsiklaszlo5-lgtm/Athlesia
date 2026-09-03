use std::collections::VecDeque;

use athlesia_arc_agi_3_adapter::bounded_episode_runtime::ArcAgi3BoundedEpisodeTermination;
use athlesia_arc_agi_3_adapter::competition_session_runtime::{
    ArcAgi3ScorecardId, ArcAgi3ScorecardRestProtocol, ArcAgi3ScorecardSummary,
};
use athlesia_arc_agi_3_adapter::ArcAgi3GameId;
use athlesia_arc_agi_3_blind_benchmark::execution_runtime::{
    ArcAgi3BlindBenchmarkExecutedEpisode, ArcAgi3BlindBenchmarkExecutionRuntime,
    ArcAgi3BlindBenchmarkExecutionStatus, ArcAgi3BlindBenchmarkHarnessEvent,
};
use athlesia_arc_agi_3_blind_benchmark::result_record::*;
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

const SOURCE_REVISION: &str = "124f278f5224169c96e5b7d854c2638a43d945a4";

const CONFIG_FINGERPRINT: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn game_id(value: &str) -> ArcAgi3GameId {
    ArcAgi3GameId::new(value.to_string()).unwrap()
}

fn card_id() -> ArcAgi3ScorecardId {
    ArcAgi3ScorecardId::new("result-card".to_string()).unwrap()
}

fn spec() -> ArcAgi3BlindBenchmarkSpec {
    ArcAgi3BlindBenchmarkSpec::new(
        ArcAgi3BlindBenchmarkRunId::new("result-run".to_string()).unwrap(),
        ArcAgi3BlindBenchmarkAgentIdentity::new(
            "athlesia".to_string(),
            "m53-result".to_string(),
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
        73,
    )
    .unwrap()
}

fn runtime() -> ArcAgi3BlindBenchmarkExecutionRuntime {
    ArcAgi3BlindBenchmarkExecutionRuntime::new(ArcAgi3BlindBenchmarkLedger::new(spec(), card_id()))
}

fn bound_run() -> ArcAgi3BlindBenchmarkBoundRun {
    ArcAgi3BlindBenchmarkRunBinding::bind(manifest(), runtime()).unwrap()
}

fn summary(score: f64) -> ArcAgi3ScorecardSummary {
    ArcAgi3ScorecardRestProtocol::decode_summary(json!({
        "card_id": "result-card",
        "score": score,
        "environments": [],
        "total_environments_completed": 2,
        "total_environments": 2,
        "total_levels_completed": 3,
        "total_levels": 3,
        "total_actions": 9,
        "competition_mode": true,
        "published_at":
            "2026-09-03T18:20:00Z",
        "audit_authority": {
            "source": "server",
            "opaque_future_field": 91
        }
    }))
    .unwrap()
}

fn finalized_bound_run(include_episodes: bool, score: f64) -> ArcAgi3BlindBenchmarkBoundRun {
    let mut bound = bound_run();

    let mut events: VecDeque<Result<ArcAgi3BlindBenchmarkHarnessEvent, &'static str>> =
        if include_episodes {
            vec![
                Ok(ArcAgi3BlindBenchmarkHarnessEvent::Episode(
                    ArcAgi3BlindBenchmarkExecutedEpisode::new(
                        game_id("blind-game-a"),
                        ArcAgi3BoundedEpisodeTermination::Won,
                        5,
                    ),
                )),
                Ok(ArcAgi3BlindBenchmarkHarnessEvent::Episode(
                    ArcAgi3BlindBenchmarkExecutedEpisode::new(
                        game_id("blind-game-b"),
                        ArcAgi3BoundedEpisodeTermination::GameOver,
                        8,
                    ),
                )),
                Ok(ArcAgi3BlindBenchmarkHarnessEvent::Finalized(summary(score))),
            ]
            .into()
        } else {
            vec![Ok(ArcAgi3BlindBenchmarkHarnessEvent::Finalized(summary(
                score,
            )))]
            .into()
        };

    {
        bound
            .runtime_mut()
            .run_with(|_| events.pop_front().expect("missing result-record event"))
            .unwrap();
    }

    assert!(events.is_empty());

    bound
}

fn faulted_bound_run() -> ArcAgi3BlindBenchmarkBoundRun {
    let mut bound = bound_run();

    {
        let result = bound
            .runtime_mut()
            .run_with(|_| -> Result<_, &'static str> { Err("forced-fault") });

        assert!(result.is_err());
    }

    bound
}

#[test]
fn result_record_rejects_ready_bound_run() {
    let bound = bound_run();

    let result = ArcAgi3BlindBenchmarkResultRecord::from_bound_run(bound);

    assert!(matches!(
        result,
        Err(ArcAgi3BlindBenchmarkResultRecordError::RuntimeNotFinalized(
            ArcAgi3BlindBenchmarkExecutionStatus::Ready,
        ))
    ));
}

#[test]
fn result_record_rejects_faulted_bound_run() {
    let bound = faulted_bound_run();

    let result = ArcAgi3BlindBenchmarkResultRecord::from_bound_run(bound);

    assert!(matches!(
        result,
        Err(ArcAgi3BlindBenchmarkResultRecordError::RuntimeNotFinalized(
            ArcAgi3BlindBenchmarkExecutionStatus::Faulted,
        ))
    ));
}

#[test]
fn finalized_zero_episode_run_creates_result_record() {
    let record =
        ArcAgi3BlindBenchmarkResultRecord::from_bound_run(finalized_bound_run(false, 2.5)).unwrap();

    assert!(record.episodes().is_empty());

    assert_eq!(record.server_score(), Some(2.5,),);
}

#[test]
fn result_record_preserves_exact_manifest_run_and_agent_identity() {
    let record =
        ArcAgi3BlindBenchmarkResultRecord::from_bound_run(finalized_bound_run(false, 1.0)).unwrap();

    assert_eq!(record.manifest().spec().run_id().as_str(), "result-run",);

    assert_eq!(record.manifest().spec().agent().name(), "athlesia",);

    assert_eq!(record.manifest().spec().agent().version(), "m53-result",);

    assert_eq!(
        record.manifest().spec().agent().source_revision(),
        SOURCE_REVISION,
    );
}

#[test]
fn result_record_preserves_exact_manifest_provenance_fields() {
    let record =
        ArcAgi3BlindBenchmarkResultRecord::from_bound_run(finalized_bound_run(false, 1.0)).unwrap();

    assert_eq!(record.manifest().harness_identity().name(), "blind-harness",);

    assert_eq!(
        record.manifest().build_identity().source_revision(),
        SOURCE_REVISION,
    );

    assert_eq!(record.manifest().protocol_identity().revision(), "2026-09",);

    assert_eq!(
        record.manifest().configuration_fingerprint().as_str(),
        CONFIG_FINGERPRINT,
    );

    assert_eq!(record.manifest().deterministic_seed(), 73,);
}

#[test]
fn result_record_preserves_exact_scorecard_identity() {
    let record =
        ArcAgi3BlindBenchmarkResultRecord::from_bound_run(finalized_bound_run(false, 3.0)).unwrap();

    assert_eq!(record.scorecard_id().as_str(), "result-card",);
}

#[test]
fn result_record_preserves_exact_server_summary_and_future_fields() {
    let record =
        ArcAgi3BlindBenchmarkResultRecord::from_bound_run(finalized_bound_run(false, 4.0)).unwrap();

    let summary = record.final_summary().unwrap();

    assert_eq!(summary.score(), 4.0,);

    assert_eq!(summary.raw()["audit_authority"]["source"], json!("server"),);

    assert_eq!(
        summary.raw()["audit_authority"]["opaque_future_field"],
        json!(91),
    );
}

#[test]
fn result_record_server_score_is_exact_server_authority() {
    let record =
        ArcAgi3BlindBenchmarkResultRecord::from_bound_run(finalized_bound_run(true, 17.25))
            .unwrap();

    assert_eq!(record.server_score(), Some(17.25,),);
}

#[test]
fn result_record_preserves_episode_order_and_observed_game_identity() {
    let record =
        ArcAgi3BlindBenchmarkResultRecord::from_bound_run(finalized_bound_run(true, 5.0)).unwrap();

    assert_eq!(record.episodes().len(), 2,);

    assert_eq!(record.episodes()[0].game_id(), &game_id("blind-game-a",),);

    assert_eq!(record.episodes()[1].game_id(), &game_id("blind-game-b",),);
}

#[test]
fn result_record_preserves_exact_episode_terminations() {
    let record =
        ArcAgi3BlindBenchmarkResultRecord::from_bound_run(finalized_bound_run(true, 5.0)).unwrap();

    assert_eq!(
        record.episodes()[0].termination(),
        ArcAgi3BoundedEpisodeTermination::Won,
    );

    assert_eq!(
        record.episodes()[1].termination(),
        ArcAgi3BoundedEpisodeTermination::GameOver,
    );
}

#[test]
fn result_record_preserves_exact_completed_cognitive_steps() {
    let record =
        ArcAgi3BlindBenchmarkResultRecord::from_bound_run(finalized_bound_run(true, 5.0)).unwrap();

    assert_eq!(record.episodes()[0].completed_cognitive_steps(), 5,);

    assert_eq!(record.episodes()[1].completed_cognitive_steps(), 8,);
}

#[test]
fn result_record_exposes_no_mutation_surface() {
    let source = include_str!("../src/result_record.rs");

    let record_start = source
        .find("pub struct ArcAgi3BlindBenchmarkResultRecord")
        .unwrap();

    let facade_start = source
        .find("pub struct UniversalArcAgi3BlindBenchmarkResultRecord")
        .unwrap();

    let record_source = &source[record_start..facade_start];

    assert!(!record_source.contains("&mut self"));

    assert!(!record_source.contains("pub fn set_"));

    assert!(!record_source.contains("pub fn update"));
}

#[test]
fn into_parts_preserves_exact_manifest_and_finalized_ledger() {
    let record =
        ArcAgi3BlindBenchmarkResultRecord::from_bound_run(finalized_bound_run(true, 6.5)).unwrap();

    let (recovered_manifest, recovered_ledger) = record.into_parts();

    assert_eq!(recovered_manifest.spec().run_id().as_str(), "result-run",);

    assert_eq!(recovered_ledger.card_id().as_str(), "result-card",);

    assert_eq!(recovered_ledger.episodes().len(), 2,);

    assert_eq!(recovered_ledger.server_score(), Some(6.5,),);
}

#[test]
fn universal_facade_matches_direct_result_record() {
    let direct =
        ArcAgi3BlindBenchmarkResultRecord::from_bound_run(finalized_bound_run(true, 9.25)).unwrap();

    let facade =
        UniversalArcAgi3BlindBenchmarkResultRecord::record(finalized_bound_run(true, 9.25))
            .unwrap();

    assert_eq!(direct.manifest(), facade.manifest(),);

    assert_eq!(direct.scorecard_id(), facade.scorecard_id(),);

    assert_eq!(direct.episodes(), facade.episodes(),);

    assert_eq!(direct.server_score(), facade.server_score(),);
}

#[test]
fn result_record_contains_only_immutable_finalized_audit_authority() {
    let source = include_str!("../src/result_record.rs");

    assert!(source.contains("RuntimeNotFinalized"));

    assert!(source.contains("ArcAgi3BlindBenchmarkRunManifest"));

    assert!(source.contains("ArcAgi3BlindBenchmarkLedger"));

    assert!(source.contains("ArcAgi3BlindBenchmarkExecutionStatus::Finalized"));

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
        "ArcAgi3BlindBenchmarkExternalHarness",
        "ArcAgi3BlindBenchmarkHarnessBridge",
        "run_with(",
        "record_episode(",
        ".finalize(",
        "open_scorecard",
        "close_scorecard",
        "begin_reset",
        ".reset(",
        "retry(",
        "sleep(",
    ] {
        assert!(
            !source.contains(forbidden,),
            "M53 result record leaked forbidden benchmark authority: {forbidden}",
        );
    }
}
