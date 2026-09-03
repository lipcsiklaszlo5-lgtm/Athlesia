use athlesia_arc_agi_3_adapter::competition_session_runtime::{
    ArcAgi3ScorecardId, ArcAgi3ScorecardRestProtocol, ArcAgi3ScorecardSummary,
};
use athlesia_arc_agi_3_blind_benchmark::execution_runtime::{
    ArcAgi3BlindBenchmarkExecutionError, ArcAgi3BlindBenchmarkExecutionRuntime,
    ArcAgi3BlindBenchmarkExecutionStatus,
};
use athlesia_arc_agi_3_blind_benchmark::run_binding::*;
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

const SOURCE_REVISION: &str = "7c0109585db3eca03c3694af67cf5a66cb1c4284";

const CONFIG_FINGERPRINT: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn card_id(value: &str) -> ArcAgi3ScorecardId {
    ArcAgi3ScorecardId::new(value.to_string()).unwrap()
}

fn spec(
    run_id: &str,
    agent_name: &str,
    agent_version: &str,
    source_revision: &str,
    budget: usize,
) -> ArcAgi3BlindBenchmarkSpec {
    ArcAgi3BlindBenchmarkSpec::new(
        ArcAgi3BlindBenchmarkRunId::new(run_id.to_string()).unwrap(),
        ArcAgi3BlindBenchmarkAgentIdentity::new(
            agent_name.to_string(),
            agent_version.to_string(),
            source_revision.to_string(),
        )
        .unwrap(),
        ArcAgi3BlindBenchmarkPolicy::new(budget).unwrap(),
    )
}

fn canonical_spec() -> ArcAgi3BlindBenchmarkSpec {
    spec("bound-run", "athlesia", "m53-binding", SOURCE_REVISION, 64)
}

fn manifest_for(spec: ArcAgi3BlindBenchmarkSpec) -> ArcAgi3BlindBenchmarkRunManifest {
    let source_revision = spec.agent().source_revision().to_string();

    ArcAgi3BlindBenchmarkRunManifest::new(
        spec,
        ArcAgi3BlindBenchmarkHarnessIdentity::new("blind-harness".to_string(), "1.0".to_string())
            .unwrap(),
        ArcAgi3BlindBenchmarkBuildIdentity::new(
            source_revision,
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
        17,
    )
    .unwrap()
}

fn ledger_for(spec: ArcAgi3BlindBenchmarkSpec) -> ArcAgi3BlindBenchmarkLedger {
    ArcAgi3BlindBenchmarkLedger::new(spec, card_id("binding-card"))
}

fn runtime_for(spec: ArcAgi3BlindBenchmarkSpec) -> ArcAgi3BlindBenchmarkExecutionRuntime {
    ArcAgi3BlindBenchmarkExecutionRuntime::new(ledger_for(spec))
}

fn server_summary() -> ArcAgi3ScorecardSummary {
    ArcAgi3ScorecardRestProtocol::decode_summary(json!({
        "card_id": "binding-card",
        "score": 0.0,
        "environments": [],
        "total_environments_completed": 0,
        "total_environments": 0,
        "total_levels_completed": 0,
        "total_levels": 0,
        "total_actions": 0,
        "competition_mode": true,
        "published_at":
            "2026-09-03T18:00:00Z"
    }))
    .unwrap()
}

#[test]
fn exact_manifest_and_pristine_runtime_bind_successfully() {
    let spec = canonical_spec();

    let manifest = manifest_for(spec.clone());

    let runtime = runtime_for(spec);

    let bound = ArcAgi3BlindBenchmarkRunBinding::bind(manifest, runtime).unwrap();

    assert_eq!(
        bound.runtime().status(),
        ArcAgi3BlindBenchmarkExecutionStatus::Ready,
    );
}

#[test]
fn non_ready_runtime_is_rejected_before_binding() {
    let spec = canonical_spec();

    let manifest = manifest_for(spec.clone());

    let mut runtime = runtime_for(spec);

    {
        let result = runtime.run_with(|_| -> Result<_, &'static str> { Err("fault-runtime") });

        assert!(matches!(
            result,
            Err(ArcAgi3BlindBenchmarkExecutionError::Harness(
                "fault-runtime"
            ))
        ));
    }

    assert_eq!(
        ArcAgi3BlindBenchmarkRunBinding::validate(&manifest, &runtime,),
        Err(ArcAgi3BlindBenchmarkRunBindingError::RuntimeNotReady(
            ArcAgi3BlindBenchmarkExecutionStatus::Faulted,
        ),),
    );
}

#[test]
fn already_finalized_ledger_is_rejected_even_if_wrapped_in_fresh_runtime() {
    let spec = canonical_spec();

    let manifest = manifest_for(spec.clone());

    let mut ledger = ledger_for(spec);

    ledger.finalize(server_summary()).unwrap();

    let runtime = ArcAgi3BlindBenchmarkExecutionRuntime::new(ledger);

    assert_eq!(
        ArcAgi3BlindBenchmarkRunBinding::validate(&manifest, &runtime,),
        Err(ArcAgi3BlindBenchmarkRunBindingError::RuntimeLedgerAlreadyFinalized,),
    );
}

#[test]
fn run_id_mismatch_is_rejected_exactly() {
    let manifest = manifest_for(canonical_spec());

    let runtime = runtime_for(spec(
        "different-run",
        "athlesia",
        "m53-binding",
        SOURCE_REVISION,
        64,
    ));

    assert_eq!(
        ArcAgi3BlindBenchmarkRunBinding::validate(&manifest, &runtime,),
        Err(ArcAgi3BlindBenchmarkRunBindingError::RunIdMismatch {
            manifest: "bound-run".to_string(),
            runtime: "different-run".to_string(),
        },),
    );
}

#[test]
fn agent_name_mismatch_is_rejected_exactly() {
    let manifest = manifest_for(canonical_spec());

    let runtime = runtime_for(spec(
        "bound-run",
        "other-agent",
        "m53-binding",
        SOURCE_REVISION,
        64,
    ));

    assert_eq!(
        ArcAgi3BlindBenchmarkRunBinding::validate(&manifest, &runtime,),
        Err(ArcAgi3BlindBenchmarkRunBindingError::AgentNameMismatch {
            manifest: "athlesia".to_string(),
            runtime: "other-agent".to_string(),
        },),
    );
}

#[test]
fn agent_version_mismatch_is_rejected_exactly() {
    let manifest = manifest_for(canonical_spec());

    let runtime = runtime_for(spec(
        "bound-run",
        "athlesia",
        "other-version",
        SOURCE_REVISION,
        64,
    ));

    assert_eq!(
        ArcAgi3BlindBenchmarkRunBinding::validate(&manifest, &runtime,),
        Err(ArcAgi3BlindBenchmarkRunBindingError::AgentVersionMismatch {
            manifest: "m53-binding".to_string(),
            runtime: "other-version".to_string(),
        },),
    );
}

#[test]
fn agent_source_revision_mismatch_is_rejected_exactly() {
    let manifest = manifest_for(canonical_spec());

    let runtime = runtime_for(spec(
        "bound-run",
        "athlesia",
        "m53-binding",
        "different-source-revision",
        64,
    ));

    assert_eq!(
        ArcAgi3BlindBenchmarkRunBinding::validate(&manifest, &runtime,),
        Err(
            ArcAgi3BlindBenchmarkRunBindingError::AgentSourceRevisionMismatch {
                manifest: SOURCE_REVISION.to_string(),
                runtime: "different-source-revision".to_string(),
            },
        ),
    );
}

#[test]
fn cognitive_step_budget_mismatch_is_rejected_exactly() {
    let manifest = manifest_for(canonical_spec());

    let runtime = runtime_for(spec(
        "bound-run",
        "athlesia",
        "m53-binding",
        SOURCE_REVISION,
        65,
    ));

    assert_eq!(
        ArcAgi3BlindBenchmarkRunBinding::validate(&manifest, &runtime,),
        Err(
            ArcAgi3BlindBenchmarkRunBindingError::EpisodeStepBudgetMismatch {
                manifest: 64,
                runtime: 65,
            },
        ),
    );
}

#[test]
fn failed_validation_does_not_mutate_runtime_or_ledger() {
    let manifest = manifest_for(canonical_spec());

    let runtime = runtime_for(spec(
        "different-run",
        "athlesia",
        "m53-binding",
        SOURCE_REVISION,
        64,
    ));

    let before_status = runtime.status();

    let before_card = runtime.ledger().card_id().as_str().to_string();

    let before_episodes = runtime.ledger().episodes().len();

    let result = ArcAgi3BlindBenchmarkRunBinding::validate(&manifest, &runtime);

    assert!(result.is_err());

    assert_eq!(runtime.status(), before_status,);

    assert_eq!(runtime.ledger().card_id().as_str(), before_card,);

    assert_eq!(runtime.ledger().episodes().len(), before_episodes,);

    assert!(runtime.ledger().final_summary().is_none());
}

#[test]
fn bound_run_preserves_exact_manifest_identity() {
    let spec = canonical_spec();

    let manifest = manifest_for(spec.clone());

    let expected_manifest = manifest.clone();

    let bound = ArcAgi3BlindBenchmarkRunBinding::bind(manifest, runtime_for(spec)).unwrap();

    assert_eq!(bound.manifest(), &expected_manifest,);
}

#[test]
fn bound_run_preserves_exact_runtime_scorecard_identity() {
    let spec = canonical_spec();

    let bound =
        ArcAgi3BlindBenchmarkRunBinding::bind(manifest_for(spec.clone()), runtime_for(spec))
            .unwrap();

    assert_eq!(bound.runtime().ledger().card_id().as_str(), "binding-card",);
}

#[test]
fn binding_does_not_change_runtime_ready_state() {
    let spec = canonical_spec();

    let bound =
        ArcAgi3BlindBenchmarkRunBinding::bind(manifest_for(spec.clone()), runtime_for(spec))
            .unwrap();

    assert_eq!(
        bound.runtime().status(),
        ArcAgi3BlindBenchmarkExecutionStatus::Ready,
    );

    assert!(bound.runtime().ledger().episodes().is_empty());

    assert!(bound.runtime().ledger().final_summary().is_none());
}

#[test]
fn into_parts_preserves_exact_manifest_and_runtime_authority() {
    let spec = canonical_spec();

    let manifest = manifest_for(spec.clone());

    let expected_manifest = manifest.clone();

    let bound = ArcAgi3BlindBenchmarkRunBinding::bind(manifest, runtime_for(spec)).unwrap();

    let (recovered_manifest, recovered_runtime) = bound.into_parts();

    assert_eq!(recovered_manifest, expected_manifest,);

    assert_eq!(
        recovered_runtime.status(),
        ArcAgi3BlindBenchmarkExecutionStatus::Ready,
    );

    assert_eq!(
        recovered_runtime.ledger().card_id().as_str(),
        "binding-card",
    );
}

#[test]
fn universal_facade_matches_direct_run_binding() {
    let spec_a = canonical_spec();

    let spec_b = canonical_spec();

    let direct =
        ArcAgi3BlindBenchmarkRunBinding::bind(manifest_for(spec_a.clone()), runtime_for(spec_a))
            .unwrap();

    let facade = UniversalArcAgi3BlindBenchmarkRunBinding::bind(
        manifest_for(spec_b.clone()),
        runtime_for(spec_b),
    )
    .unwrap();

    assert_eq!(direct.manifest(), facade.manifest(),);

    assert_eq!(
        direct.runtime().ledger().card_id(),
        facade.runtime().ledger().card_id(),
    );

    assert_eq!(direct.runtime().status(), facade.runtime().status(),);
}

#[test]
fn run_binding_contains_only_identity_and_pristine_runtime_authority() {
    let source = include_str!("../src/run_binding.rs");

    assert!(source.contains("RuntimeHasEpisodeHistory"));

    assert!(source.contains("RuntimeLedgerAlreadyFinalized"));

    assert!(source.contains("RunIdMismatch"));

    assert!(source.contains("AgentSourceRevisionMismatch"));

    assert!(source.contains("EpisodeStepBudgetMismatch"));

    assert!(source.contains(".episodes()"));

    assert!(source.contains(".final_summary()"));

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
            "M53 run binding leaked forbidden benchmark authority: {forbidden}",
        );
    }
}
