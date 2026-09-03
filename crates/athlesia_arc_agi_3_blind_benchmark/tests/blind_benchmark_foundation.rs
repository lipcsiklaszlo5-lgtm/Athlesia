use athlesia_arc_agi_3_adapter::bounded_episode_runtime::ArcAgi3BoundedEpisodeTermination;
use athlesia_arc_agi_3_adapter::competition_session_runtime::{
    ArcAgi3ScorecardId, ArcAgi3ScorecardRestProtocol,
};
use athlesia_arc_agi_3_adapter::ArcAgi3GameId;
use athlesia_arc_agi_3_blind_benchmark::*;
use serde_json::json;

fn run_id() -> ArcAgi3BlindBenchmarkRunId {
    ArcAgi3BlindBenchmarkRunId::new("blind-run-001".to_string()).unwrap()
}

fn agent() -> ArcAgi3BlindBenchmarkAgentIdentity {
    ArcAgi3BlindBenchmarkAgentIdentity::new(
        "athlesia".to_string(),
        "m53-foundation".to_string(),
        "66bcf980dfc7a443aea0d513b10142e7d8533ef5".to_string(),
    )
    .unwrap()
}

fn policy() -> ArcAgi3BlindBenchmarkPolicy {
    ArcAgi3BlindBenchmarkPolicy::new(64).unwrap()
}

fn spec() -> ArcAgi3BlindBenchmarkSpec {
    ArcAgi3BlindBenchmarkSpec::new(run_id(), agent(), policy())
}

fn card_id(value: &str) -> ArcAgi3ScorecardId {
    ArcAgi3ScorecardId::new(value.to_string()).unwrap()
}

fn game_id(value: &str) -> ArcAgi3GameId {
    ArcAgi3GameId::new(value.to_string()).unwrap()
}

fn summary(
    card: &str,
    score: f64,
    competition_mode: bool,
) -> athlesia_arc_agi_3_adapter::competition_session_runtime::ArcAgi3ScorecardSummary {
    ArcAgi3ScorecardRestProtocol::decode_summary(json!({
        "card_id": card,
        "score": score,
        "environments": [],
        "total_environments_completed": 2,
        "total_environments": 3,
        "total_levels_completed": 7,
        "total_levels": 11,
        "total_actions": 89,
        "competition_mode":
            competition_mode,
        "published_at":
            "2026-09-03T16:00:00Z",
        "opaque": {
            "run": "blind-run-001"
        },
        "future_server_field": {
            "preserve": true
        }
    }))
    .unwrap()
}

#[test]
fn blind_run_id_rejects_empty_identity() {
    assert_eq!(
        ArcAgi3BlindBenchmarkRunId::new("   ".to_string(),),
        Err(ArcAgi3BlindBenchmarkFoundationError::EmptyRunId,),
    );
}

#[test]
fn agent_identity_requires_name_version_and_source_revision() {
    assert_eq!(
        ArcAgi3BlindBenchmarkAgentIdentity::new(
            "".to_string(),
            "v1".to_string(),
            "abc".to_string(),
        ),
        Err(ArcAgi3BlindBenchmarkFoundationError::EmptyAgentName,),
    );

    assert_eq!(
        ArcAgi3BlindBenchmarkAgentIdentity::new(
            "athlesia".to_string(),
            " ".to_string(),
            "abc".to_string(),
        ),
        Err(ArcAgi3BlindBenchmarkFoundationError::EmptyAgentVersion,),
    );

    assert_eq!(
        ArcAgi3BlindBenchmarkAgentIdentity::new(
            "athlesia".to_string(),
            "v1".to_string(),
            "".to_string(),
        ),
        Err(ArcAgi3BlindBenchmarkFoundationError::EmptySourceRevision,),
    );
}

#[test]
fn blind_policy_requires_positive_episode_step_budget() {
    assert_eq!(ArcAgi3BlindBenchmarkPolicy::new(0,), None,);

    assert_eq!(
        ArcAgi3BlindBenchmarkPolicy::new(64,)
            .unwrap()
            .max_cognitive_steps_per_episode(),
        64,
    );
}

#[test]
fn blind_spec_preserves_only_run_agent_and_budget_authority() {
    let spec = spec();

    assert_eq!(spec.run_id().as_str(), "blind-run-001",);

    assert_eq!(spec.agent().name(), "athlesia",);

    assert_eq!(spec.agent().version(), "m53-foundation",);

    assert_eq!(
        spec.agent().source_revision(),
        "66bcf980dfc7a443aea0d513b10142e7d8533ef5",
    );

    assert_eq!(spec.policy().max_cognitive_steps_per_episode(), 64,);
}

#[test]
fn ledger_preserves_exact_server_scorecard_identity() {
    let ledger = ArcAgi3BlindBenchmarkLedger::new(spec(), card_id("server-card-001"));

    assert_eq!(ledger.card_id().as_str(), "server-card-001",);

    assert_eq!(ledger.status(), ArcAgi3BlindBenchmarkStatus::Recording,);

    assert!(ledger.final_summary().is_none());
}

#[test]
fn episode_observation_preserves_exact_game_outcome_and_step_count() {
    let mut ledger = ArcAgi3BlindBenchmarkLedger::new(spec(), card_id("card"));

    let observed = ledger
        .record_episode(
            game_id("hidden-game-a"),
            ArcAgi3BoundedEpisodeTermination::Won,
            17,
        )
        .unwrap();

    assert_eq!(observed.episode_index(), 0,);

    assert_eq!(observed.game_id(), &game_id("hidden-game-a"),);

    assert_eq!(
        observed.termination(),
        ArcAgi3BoundedEpisodeTermination::Won,
    );

    assert_eq!(observed.completed_cognitive_steps(), 17,);
}

#[test]
fn episode_history_is_append_only_and_preserves_execution_order() {
    let mut ledger = ArcAgi3BlindBenchmarkLedger::new(spec(), card_id("card"));

    ledger
        .record_episode(
            game_id("game-a"),
            ArcAgi3BoundedEpisodeTermination::GameOver,
            9,
        )
        .unwrap();

    ledger
        .record_episode(
            game_id("game-b"),
            ArcAgi3BoundedEpisodeTermination::StepBudgetExhausted,
            64,
        )
        .unwrap();

    assert_eq!(ledger.episodes().len(), 2,);

    assert_eq!(ledger.episodes()[0].episode_index(), 0,);

    assert_eq!(ledger.episodes()[1].episode_index(), 1,);

    assert_eq!(ledger.episodes()[0].game_id(), &game_id("game-a"),);

    assert_eq!(ledger.episodes()[1].game_id(), &game_id("game-b"),);
}

#[test]
fn finalization_preserves_exact_server_score_without_recalculation() {
    let mut ledger = ArcAgi3BlindBenchmarkLedger::new(spec(), card_id("score-card"));

    ledger
        .finalize(summary("score-card", 37.125, true))
        .unwrap();

    assert_eq!(ledger.server_score(), Some(37.125),);

    assert_eq!(ledger.status(), ArcAgi3BlindBenchmarkStatus::Finalized,);

    assert_eq!(
        ledger.final_summary().unwrap().raw()["future_server_field"]["preserve"],
        json!(true),
    );
}

#[test]
fn mismatched_server_card_is_rejected_transactionally() {
    let mut ledger = ArcAgi3BlindBenchmarkLedger::new(spec(), card_id("expected-card"));

    let error = ledger
        .finalize(summary("wrong-card", 1.0, true))
        .unwrap_err();

    assert_eq!(
        error,
        ArcAgi3BlindBenchmarkFoundationError::CardIdentityMismatch {
            expected: "expected-card".to_string(),
            observed: "wrong-card".to_string(),
        },
    );

    assert_eq!(ledger.status(), ArcAgi3BlindBenchmarkStatus::Recording,);

    assert!(ledger.final_summary().is_none());
}

#[test]
fn noncompetition_server_summary_is_rejected_transactionally() {
    let mut ledger = ArcAgi3BlindBenchmarkLedger::new(spec(), card_id("card"));

    assert_eq!(
        ledger.finalize(summary("card", 1.0, false,),),
        Err(ArcAgi3BlindBenchmarkFoundationError::CompetitionModeMismatch,),
    );

    assert_eq!(ledger.status(), ArcAgi3BlindBenchmarkStatus::Recording,);
}

#[test]
fn finalized_benchmark_rejects_further_episode_observations() {
    let mut ledger = ArcAgi3BlindBenchmarkLedger::new(spec(), card_id("card"));

    ledger.finalize(summary("card", 10.0, true)).unwrap();

    assert_eq!(
        ledger.record_episode(
            game_id("late-game"),
            ArcAgi3BoundedEpisodeTermination::Won,
            1,
        ),
        Err(ArcAgi3BlindBenchmarkFoundationError::BenchmarkAlreadyFinalized,),
    );

    assert!(ledger.episodes().is_empty());
}

#[test]
fn finalized_benchmark_rejects_second_server_summary() {
    let mut ledger = ArcAgi3BlindBenchmarkLedger::new(spec(), card_id("card"));

    ledger.finalize(summary("card", 10.0, true)).unwrap();

    assert_eq!(
        ledger.finalize(summary("card", 20.0, true,),),
        Err(ArcAgi3BlindBenchmarkFoundationError::BenchmarkAlreadyFinalized,),
    );

    assert_eq!(ledger.server_score(), Some(10.0),);
}

#[test]
fn universal_facade_matches_direct_foundation_construction() {
    let direct = ArcAgi3BlindBenchmarkLedger::new(spec(), card_id("card"));

    let facade = UniversalArcAgi3BlindBenchmarkFoundation::ledger(spec(), card_id("card"));

    assert_eq!(direct.spec(), facade.spec(),);

    assert_eq!(direct.card_id(), facade.card_id(),);

    assert_eq!(direct.status(), facade.status(),);
}

#[test]
fn foundation_contains_no_score_formula_action_policy_or_network_transport() {
    let source = include_str!("../src/lib.rs");

    for forbidden in [
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
        "open_scorecard",
        "close_scorecard",
        "execute(",
        "begin_reset",
        ".reset(",
        "retry(",
        "sleep(",
    ] {
        assert!(
            !source.contains(forbidden,),
            "M53 foundation leaked forbidden execution or scoring authority: {forbidden}",
        );
    }
}

#[test]
fn foundation_has_no_predeclared_environment_catalog_or_hidden_task_manifest() {
    let source = include_str!("../src/lib.rs");

    for forbidden in [
        "game_catalog",
        "environment_catalog",
        "hidden_games",
        "hidden_environments",
        "evaluation_games",
        "public_games",
        "game_ids:",
        "environment_ids:",
    ] {
        assert!(
            !source.contains(forbidden,),
            "M53 foundation leaked predeclared benchmark environment knowledge: {forbidden}",
        );
    }

    assert!(source.contains("record_episode"));

    assert!(source.contains("ArcAgi3ScorecardSummary"));
}
