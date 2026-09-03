use athlesia_arc_agi_3_adapter::bounded_episode_runtime::ArcAgi3BoundedEpisodeTermination;
use athlesia_arc_agi_3_adapter::competition_session_runtime::{
    ArcAgi3ScorecardId, ArcAgi3ScorecardRestProtocol, ArcAgi3ScorecardSummary,
};
use athlesia_arc_agi_3_adapter::ArcAgi3GameId;
use athlesia_arc_agi_3_blind_benchmark::execution_runtime::*;
use athlesia_arc_agi_3_blind_benchmark::*;
use serde_json::json;

fn run_id() -> ArcAgi3BlindBenchmarkRunId {
    ArcAgi3BlindBenchmarkRunId::new("blind-execution-run".to_string()).unwrap()
}

fn agent() -> ArcAgi3BlindBenchmarkAgentIdentity {
    ArcAgi3BlindBenchmarkAgentIdentity::new(
        "athlesia".to_string(),
        "m53-execution".to_string(),
        "09767028ebc8ef9c8ee8393fa77c083289378bbf".to_string(),
    )
    .unwrap()
}

fn policy() -> ArcAgi3BlindBenchmarkPolicy {
    ArcAgi3BlindBenchmarkPolicy::new(8).unwrap()
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

fn ledger() -> ArcAgi3BlindBenchmarkLedger {
    ArcAgi3BlindBenchmarkLedger::new(spec(), card_id("blind-card"))
}

fn summary(card: &str, score: f64, competition_mode: bool) -> ArcAgi3ScorecardSummary {
    ArcAgi3ScorecardRestProtocol::decode_summary(json!({
        "card_id": card,
        "score": score,
        "environments": [],
        "total_environments_completed": 2,
        "total_environments": 2,
        "total_levels_completed": 5,
        "total_levels": 5,
        "total_actions": 11,
        "competition_mode":
            competition_mode,
        "published_at":
            "2026-09-03T17:30:00Z",
        "future_server_field": {
            "authoritative": true
        }
    }))
    .unwrap()
}

fn episode(
    game: &str,
    termination: ArcAgi3BoundedEpisodeTermination,
    steps: usize,
) -> ArcAgi3BlindBenchmarkHarnessEvent {
    ArcAgi3BlindBenchmarkHarnessEvent::Episode(ArcAgi3BlindBenchmarkExecutedEpisode::new(
        game_id(game),
        termination,
        steps,
    ))
}

#[test]
fn first_harness_request_contains_only_zero_index_and_exact_step_budget() {
    let mut runtime = ArcAgi3BlindBenchmarkExecutionRuntime::new(ledger());

    let mut calls = 0;

    runtime
        .run_with::<(), _>(|request| {
            assert_eq!(request.episode_index(), 0,);

            assert_eq!(request.max_cognitive_steps_per_episode(), 8,);

            calls += 1;

            Ok(ArcAgi3BlindBenchmarkHarnessEvent::Finalized(summary(
                "blind-card",
                1.0,
                true,
            )))
        })
        .unwrap();

    assert_eq!(calls, 1,);
}

#[test]
fn runtime_records_unknown_environment_identities_only_after_harness_execution() {
    let mut runtime = ArcAgi3BlindBenchmarkExecutionRuntime::new(ledger());

    let mut calls = 0;

    runtime
        .run_with::<(), _>(|_| {
            let event = match calls {
                0 => episode("unknown-a", ArcAgi3BoundedEpisodeTermination::GameOver, 3),
                1 => episode("unknown-b", ArcAgi3BoundedEpisodeTermination::Won, 5),
                _ => {
                    ArcAgi3BlindBenchmarkHarnessEvent::Finalized(summary("blind-card", 7.25, true))
                }
            };

            calls += 1;

            Ok(event)
        })
        .unwrap();

    let episodes = runtime.ledger().episodes();

    assert_eq!(episodes.len(), 2,);

    assert_eq!(episodes[0].game_id(), &game_id("unknown-a"),);

    assert_eq!(episodes[1].game_id(), &game_id("unknown-b"),);
}

#[test]
fn harness_requests_advance_only_from_completed_episode_history() {
    let mut runtime = ArcAgi3BlindBenchmarkExecutionRuntime::new(ledger());

    let mut observed = Vec::new();

    runtime
        .run_with::<(), _>(|request| {
            observed.push(request.episode_index());

            Ok(match request.episode_index() {
                0 => episode("game-a", ArcAgi3BoundedEpisodeTermination::GameOver, 1),
                1 => episode("game-b", ArcAgi3BoundedEpisodeTermination::Won, 2),
                _ => ArcAgi3BlindBenchmarkHarnessEvent::Finalized(summary("blind-card", 2.0, true)),
            })
        })
        .unwrap();

    assert_eq!(observed, vec![0, 1, 2,],);
}

#[test]
fn exact_episode_step_budget_is_accepted() {
    let mut runtime = ArcAgi3BlindBenchmarkExecutionRuntime::new(ledger());

    let mut calls = 0;

    runtime
        .run_with::<(), _>(|_| {
            let event = if calls == 0 {
                episode(
                    "budget-edge",
                    ArcAgi3BoundedEpisodeTermination::StepBudgetExhausted,
                    8,
                )
            } else {
                ArcAgi3BlindBenchmarkHarnessEvent::Finalized(summary("blind-card", 1.0, true))
            };

            calls += 1;

            Ok(event)
        })
        .unwrap();

    assert_eq!(
        runtime.ledger().episodes()[0].completed_cognitive_steps(),
        8,
    );
}

#[test]
fn episode_over_step_budget_faults_before_ledger_commit() {
    let mut runtime = ArcAgi3BlindBenchmarkExecutionRuntime::new(ledger());

    let result = runtime.run_with::<(), _>(|_| {
        Ok(episode(
            "over-budget",
            ArcAgi3BoundedEpisodeTermination::StepBudgetExhausted,
            9,
        ))
    });

    assert!(matches!(
        result,
        Err(
            ArcAgi3BlindBenchmarkExecutionError::EpisodeStepBudgetExceeded {
                episode_index: 0,
                observed_steps: 9,
                maximum_steps: 8,
            }
        )
    ));

    assert!(runtime.ledger().episodes().is_empty());

    assert_eq!(
        runtime.status(),
        ArcAgi3BlindBenchmarkExecutionStatus::Faulted,
    );
}

#[test]
fn harness_failure_faults_runtime_without_fabricating_episode() {
    let mut runtime = ArcAgi3BlindBenchmarkExecutionRuntime::new(ledger());

    let result = runtime.run_with(
        |_| -> Result<ArcAgi3BlindBenchmarkHarnessEvent, &'static str> { Err("harness failed") },
    );

    assert!(matches!(
        result,
        Err(ArcAgi3BlindBenchmarkExecutionError::Harness(
            "harness failed"
        ))
    ));

    assert!(runtime.ledger().episodes().is_empty());

    assert_eq!(
        runtime.status(),
        ArcAgi3BlindBenchmarkExecutionStatus::Faulted,
    );
}

#[test]
fn faulted_runtime_never_retries_harness_on_second_run() {
    let mut runtime = ArcAgi3BlindBenchmarkExecutionRuntime::new(ledger());

    let mut first_calls = 0;

    let _ = runtime.run_with(
        |_| -> Result<ArcAgi3BlindBenchmarkHarnessEvent, &'static str> {
            first_calls += 1;

            Err("indeterminate external failure")
        },
    );

    assert_eq!(first_calls, 1,);

    let mut second_calls = 0;

    let second = runtime.run_with::<(), _>(|_| {
        second_calls += 1;

        Ok(ArcAgi3BlindBenchmarkHarnessEvent::Finalized(summary(
            "blind-card",
            99.0,
            true,
        )))
    });

    assert!(matches!(
        second,
        Err(ArcAgi3BlindBenchmarkExecutionError::RuntimeNotReady(
            ArcAgi3BlindBenchmarkExecutionStatus::Faulted
        ))
    ));

    assert_eq!(second_calls, 0,);
}

#[test]
fn server_score_is_preserved_without_local_recalculation() {
    let mut runtime = ArcAgi3BlindBenchmarkExecutionRuntime::new(ledger());

    runtime
        .run_with::<(), _>(|_| {
            Ok(ArcAgi3BlindBenchmarkHarnessEvent::Finalized(summary(
                "blind-card",
                37.125,
                true,
            )))
        })
        .unwrap();

    assert_eq!(runtime.ledger().server_score(), Some(37.125),);

    assert_eq!(
        runtime.status(),
        ArcAgi3BlindBenchmarkExecutionStatus::Finalized,
    );
}

#[test]
fn zero_episode_server_finalization_is_preserved_without_fabrication() {
    let mut runtime = ArcAgi3BlindBenchmarkExecutionRuntime::new(ledger());

    runtime
        .run_with::<(), _>(|_| {
            Ok(ArcAgi3BlindBenchmarkHarnessEvent::Finalized(summary(
                "blind-card",
                0.0,
                true,
            )))
        })
        .unwrap();

    assert!(runtime.ledger().episodes().is_empty());

    assert_eq!(runtime.ledger().server_score(), Some(0.0),);
}

#[test]
fn mismatched_final_scorecard_faults_without_rewriting_identity() {
    let mut runtime = ArcAgi3BlindBenchmarkExecutionRuntime::new(ledger());

    let result = runtime.run_with::<(), _>(|_| {
        Ok(ArcAgi3BlindBenchmarkHarnessEvent::Finalized(summary(
            "wrong-card",
            10.0,
            true,
        )))
    });

    assert!(matches!(
        result,
        Err(ArcAgi3BlindBenchmarkExecutionError::Foundation(
            ArcAgi3BlindBenchmarkFoundationError::CardIdentityMismatch { .. }
        ))
    ));

    assert_eq!(runtime.ledger().card_id().as_str(), "blind-card",);

    assert!(runtime.ledger().final_summary().is_none());

    assert_eq!(
        runtime.status(),
        ArcAgi3BlindBenchmarkExecutionStatus::Faulted,
    );
}

#[test]
fn noncompetition_final_summary_faults_transactionally() {
    let mut runtime = ArcAgi3BlindBenchmarkExecutionRuntime::new(ledger());

    let result = runtime.run_with::<(), _>(|_| {
        Ok(ArcAgi3BlindBenchmarkHarnessEvent::Finalized(summary(
            "blind-card",
            5.0,
            false,
        )))
    });

    assert!(matches!(
        result,
        Err(ArcAgi3BlindBenchmarkExecutionError::Foundation(
            ArcAgi3BlindBenchmarkFoundationError::CompetitionModeMismatch
        ))
    ));

    assert!(runtime.ledger().final_summary().is_none());

    assert_eq!(
        runtime.status(),
        ArcAgi3BlindBenchmarkExecutionStatus::Faulted,
    );
}

#[test]
fn finalized_runtime_blocks_second_execution_without_harness_call() {
    let mut runtime = ArcAgi3BlindBenchmarkExecutionRuntime::new(ledger());

    runtime
        .run_with::<(), _>(|_| {
            Ok(ArcAgi3BlindBenchmarkHarnessEvent::Finalized(summary(
                "blind-card",
                1.0,
                true,
            )))
        })
        .unwrap();

    let mut calls = 0;

    let result = runtime.run_with::<(), _>(|_| {
        calls += 1;

        Ok(ArcAgi3BlindBenchmarkHarnessEvent::Finalized(summary(
            "blind-card",
            2.0,
            true,
        )))
    });

    assert!(matches!(
        result,
        Err(ArcAgi3BlindBenchmarkExecutionError::RuntimeNotReady(
            ArcAgi3BlindBenchmarkExecutionStatus::Finalized
        ))
    ));

    assert_eq!(calls, 0,);

    assert_eq!(runtime.ledger().server_score(), Some(1.0),);
}

#[test]
fn into_ledger_preserves_exact_execution_history_and_final_summary() {
    let mut runtime = ArcAgi3BlindBenchmarkExecutionRuntime::new(ledger());

    let mut calls = 0;

    runtime
        .run_with::<(), _>(|_| {
            let event = if calls == 0 {
                episode("observed-game", ArcAgi3BoundedEpisodeTermination::Won, 4)
            } else {
                ArcAgi3BlindBenchmarkHarnessEvent::Finalized(summary("blind-card", 12.5, true))
            };

            calls += 1;

            Ok(event)
        })
        .unwrap();

    let ledger = runtime.into_ledger();

    assert_eq!(ledger.episodes().len(), 1,);

    assert_eq!(ledger.episodes()[0].game_id(), &game_id("observed-game"),);

    assert_eq!(ledger.server_score(), Some(12.5),);
}

#[test]
fn universal_facade_matches_direct_execution_runtime() {
    let mut direct = ArcAgi3BlindBenchmarkExecutionRuntime::new(ledger());

    let mut facade = UniversalArcAgi3BlindBenchmarkExecutionRuntime::runtime(ledger());

    direct
        .run_with::<(), _>(|_| {
            Ok(ArcAgi3BlindBenchmarkHarnessEvent::Finalized(summary(
                "blind-card",
                3.0,
                true,
            )))
        })
        .unwrap();

    UniversalArcAgi3BlindBenchmarkExecutionRuntime::run_with::<(), _>(&mut facade, |_| {
        Ok(ArcAgi3BlindBenchmarkHarnessEvent::Finalized(summary(
            "blind-card",
            3.0,
            true,
        )))
    })
    .unwrap();

    assert_eq!(direct.status(), facade.status(),);

    assert_eq!(
        direct.ledger().server_score(),
        facade.ledger().server_score(),
    );
}

#[test]
fn execution_runtime_contains_no_hidden_catalog_score_formula_action_policy_network_or_retry() {
    let source = include_str!("../src/execution_runtime.rs");

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
        "open_scorecard",
        "close_scorecard",
        "begin_reset",
        ".reset(",
        "retry(",
        "sleep(",
    ] {
        assert!(
            !source.contains(forbidden,),
            "M53 execution runtime leaked forbidden benchmark authority: {forbidden}",
        );
    }

    assert!(source.contains("FnMut"));

    assert!(source.contains("ArcAgi3BlindBenchmarkHarnessEvent"));

    assert!(source.contains("ArcAgi3ScorecardSummary"));
}
