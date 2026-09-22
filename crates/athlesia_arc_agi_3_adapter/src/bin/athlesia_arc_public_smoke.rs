use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
};

use athlesia_arc_agi_3_adapter::{
    cognitive_trace::ArcAgi3JsonlTraceSink,
    environment_transport_boundary::ArcAgi3RestTransport,
    live_environment_runtime::ArcAgi3LiveEnvironmentRuntime,
    production_successor_runtime::ArcAgi3ProductionSuccessorPolicy,
    successor_episode_runtime::{ArcAgi3SuccessorEpisodePolicy, ArcAgi3SuccessorEpisodeRuntime},
    ArcAgi3GameId,
};

use athlesia_autonomous_active_experimentation::{
    EmpiricalExpectedEpistemicProgressPolicy, EpistemicForecastDiscriminationPolicy,
};

use athlesia_executive_agency::{
    ExecutiveAgencyPolicy, ExecutiveGoal, ExecutiveSelectionThresholds, ExecutiveUtilityWeights,
};

use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

use athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy;

use reqwest::blocking::Client;
use serde_json::{json, Value};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).expect("diagnostic policy uses bounded positive signal")
}

fn inert_goal() -> ExecutiveGoal {
    /*
     * Structural placeholder only.
     *
     * Production policy below supplies goal_alignment = ZERO.
     * Therefore this goal cannot authorize exploitation.
     *
     * The real run is intentionally driven only by:
     *   bootstrap ignorance,
     *   retained epistemic evidence,
     *   grounded exact-state ignorance.
     */
    ExecutiveGoal::new(
        CognitiveStructure::atom(0x4154_484C_4553_4941),
        signal(900),
        CognitiveSignal::zero(),
    )
}

fn production_policy<'a>(goal: &'a ExecutiveGoal) -> ArcAgi3ProductionSuccessorPolicy<'a> {
    let version_policy = GroundedExplanatoryVersionSpacePolicy::new(1, 64, 512, 256).unwrap();

    let discrimination_policy = EpistemicForecastDiscriminationPolicy::new(512, 512).unwrap();

    let expectation_policy = EmpiricalExpectedEpistemicProgressPolicy::new(256, 256, 1).unwrap();

    let executive_policy = ExecutiveAgencyPolicy::new(
        1,
        8,
        16,
        1,
        ExecutiveUtilityWeights::new(0, 0, 0, 1000, 0).unwrap(),
        ExecutiveSelectionThresholds::new(
            signal(100),
            signal(100),
            signal(1),
            signal(600),
            signal(100),
        )
        .unwrap(),
    )
    .unwrap();

    ArcAgi3ProductionSuccessorPolicy::new(
        goal,
        /*
         * CRITICAL:
         * disable externally supplied goal exploitation.
         */
        CognitiveSignal::zero(),
        signal(100),
        version_policy,
        discrimination_policy,
        expectation_policy,
        executive_policy,
        signal(900),
    )
    .unwrap()
}

fn open_research_scorecard(
    client: &Client,
    base_url: &str,
    api_key: &str,
) -> Result<String, String> {
    /*
     * Do NOT send competition_mode.
     *
     * Official ARC server interprets presence of that field as
     * competition mode. This runner is repeatable public research.
     */
    let response = client
        .post(format!(
            "{}/api/scorecard/open",
            base_url.trim_end_matches('/'),
        ))
        .header("X-API-Key", api_key)
        .json(&json!({
            "tags": [
                "agent",
                "athlesia",
                "goal-neutral",
                "diagnostic"
            ],
            "opaque": {
                "runner": "athlesia_arc_public_smoke",
                "goal_authority": "disabled"
            }
        }))
        .send()
        .map_err(|error| format!("scorecard open transport failed: {error}"))?;

    let response = response
        .error_for_status()
        .map_err(|error| format!("scorecard open rejected: {error}"))?;

    let value: Value = response
        .json()
        .map_err(|error| format!("scorecard open decode failed: {error}"))?;

    value
        .get("card_id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("scorecard open response missing card_id: {value}"))
}

fn close_research_scorecard(
    client: &Client,
    base_url: &str,
    api_key: &str,
    card_id: &str,
) -> Result<Value, String> {
    let response = client
        .post(format!(
            "{}/api/scorecard/close",
            base_url.trim_end_matches('/'),
        ))
        .header("X-API-Key", api_key)
        .json(&json!({
            "card_id": card_id,
        }))
        .send()
        .map_err(|error| format!("scorecard close transport failed: {error}"))?;

    let response = response
        .error_for_status()
        .map_err(|error| format!("scorecard close rejected: {error}"))?;

    response
        .json()
        .map_err(|error| format!("scorecard close decode failed: {error}"))
}

fn cognitive_snapshot(
    runtime: &ArcAgi3LiveEnvironmentRuntime<ArcAgi3RestTransport>,
    version_policy: GroundedExplanatoryVersionSpacePolicy,
) -> Value {
    let cognitive_runtime = runtime.cognitive_runtime();

    let observation = cognitive_runtime.observation();

    let cognition = cognitive_runtime.cognition();

    let grounded = cognitive_runtime.current_grounded_world_state();

    let version_space = cognition.current_explanatory_version_space(version_policy);

    let active_hypotheses = version_space
        .active()
        .iter()
        .take(32)
        .map(|hypothesis| {
            json!({
                "transformation":
                    format!("{:?}",
                        hypothesis.transformation()),
                "context":
                    format!("{:?}",
                        hypothesis.context()),
                "effect_kind":
                    format!("{:?}",
                        hypothesis.effect_kind()),
                "effect_fact":
                    format!("{:?}",
                        hypothesis.effect_fact()),
                "support":
                    hypothesis.support_count(),
                "opportunities":
                    hypothesis.opportunity_count(),
                "counterexamples":
                    hypothesis.counterexample_count(),
            })
        })
        .collect::<Vec<_>>();

    json!({
        "runtime_status":
            format!("{:?}", runtime.status()),

        "game_state":
            format!("{:?}", observation.state()),

        "levels_completed":
            observation.levels_completed(),

        "win_levels":
            observation.win_levels(),

        "available_actions":
            observation
                .available_actions()
                .actions()
                .iter()
                .map(|action| format!("{action:?}"))
                .collect::<Vec<_>>(),

        "strict_grounded":
            grounded.is_some(),

        "grounded_fact_count":
            grounded
                .as_ref()
                .map(|state| state.fact_count()),

        "bootstrap_action_events":
            cognition
                .bootstrap_action_coverage_event_count(),

        "perceptual_temporal_records":
            cognition
                .perceptual_temporal_record_count(),

        "grouping_behavior_records":
            cognition
                .perceptual_grouping_behavior_record_count(),

        "grouping_appearance_records":
            cognition
                .perceptual_grouping_appearance_record_count(),

        "transition_episodes":
            cognition.transition_episode_count(),

        "active_hypotheses":
            version_space.active_count(),

        "epistemic_progress_events":
            cognition.epistemic_progress_event_count(),

        "epistemic_transfer_events":
            cognition.epistemic_transfer_progress_event_count(),

        "hypotheses":
            active_hypotheses,
    })
}

fn run() -> Result<(), String> {
    let base_url =
        env::var("ATHLESIA_ARC_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8001".to_string());

    let api_key = env::var("ATHLESIA_ARC_API_KEY").unwrap_or_else(|_| "1234".to_string());

    let game_name = env::var("ATHLESIA_ARC_GAME_ID").unwrap_or_else(|_| "ls20".to_string());

    let max_steps = env::var("ATHLESIA_ARC_MAX_STEPS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(64);

    if max_steps == 0 {
        return Err("ATHLESIA_ARC_MAX_STEPS must be positive".to_string());
    }

    let trace_path = env::var("ATHLESIA_ARC_TRACE")
        .unwrap_or_else(|_| "/tmp/athlesia_arc_trace.jsonl".to_string());

    let diag_path = env::var("ATHLESIA_ARC_DIAG")
        .unwrap_or_else(|_| "/tmp/athlesia_arc_diag.jsonl".to_string());

    let scorecard_path = env::var("ATHLESIA_ARC_SCORECARD")
        .unwrap_or_else(|_| "/tmp/athlesia_arc_scorecard.json".to_string());

    let client = Client::builder()
        .cookie_store(true)
        .build()
        .map_err(|error| error.to_string())?;

    let card_id = open_research_scorecard(&client, &base_url, &api_key)?;

    println!("GAME={game_name}");
    println!("CARD_ID={card_id}");
    println!("MAX_STEPS={max_steps}");
    println!("GOAL_AUTHORITY=DISABLED");
    println!("TRACE={trace_path}");
    println!("DIAG={diag_path}");

    let game_id = ArcAgi3GameId::new(game_name.clone())
        .ok_or_else(|| format!("invalid game id: {game_name}"))?;

    let transport = ArcAgi3RestTransport::new(base_url.clone(), api_key.clone())
        .map_err(|error| format!("REST transport init failed: {error:?}"))?;

    let mut runtime =
        ArcAgi3LiveEnvironmentRuntime::start(transport, &game_id, &card_id, 1_000_000)
            .map_err(|error| format!("game start failed: {error:?}"))?;

    let trace_file = File::create(&trace_path).map_err(|error| error.to_string())?;

    let mut trace_sink = ArcAgi3JsonlTraceSink::new(BufWriter::new(trace_file));

    let diag_file = File::create(&diag_path).map_err(|error| error.to_string())?;

    let mut diag_writer = BufWriter::new(diag_file);

    let goal = inert_goal();

    let policy = production_policy(&goal);

    let version_policy = policy.version_policy();

    let episode_policy = ArcAgi3SuccessorEpisodePolicy::new(max_steps, max_steps)
        .ok_or_else(|| "invalid episode budget".to_string())?;

    let mut attempt = 0_usize;

    let episode =
        ArcAgi3SuccessorEpisodeRuntime::run_with(&mut runtime, episode_policy, |runtime| {
            attempt += 1;

            let before = cognitive_snapshot(runtime, version_policy);

            let result = runtime
                .execute_production_evidence_faithful_successor_with_trace(policy, &mut trace_sink);

            let after = cognitive_snapshot(runtime, version_policy);

            let decision = match &result {
                Ok(Some(step)) => {
                    json!({
                        "kind": "executed",

                        "authority":
                            format!("{:?}",
                                step.authority()
                                    .kind()),

                        "action":
                            format!("{:?}",
                                step.action()),

                        "cognitive_action":
                            format!("{:?}",
                                step.cognitive_action()),
                    })
                }

                Ok(None) => {
                    json!({
                        "kind": "abstained",
                    })
                }

                Err(error) => {
                    json!({
                        "kind": "error",
                        "error":
                            format!("{error:?}"),
                    })
                }
            };

            let record = json!({
                "attempt": attempt,
                "before": before,
                "decision": decision,
                "after": after,
            });

            if let Err(error) = serde_json::to_writer(&mut diag_writer, &record) {
                eprintln!("DIAGNOSTIC_WRITE_ERROR: {error}");
            } else {
                let _ = diag_writer.write_all(b"\n");

                let _ = diag_writer.flush();
            }

            result
        });

    let _ = trace_sink.writer.flush();
    let _ = diag_writer.flush();

    let final_snapshot = cognitive_snapshot(&runtime, version_policy);

    println!(
        "FINAL_COGNITION={}",
        serde_json::to_string(&final_snapshot).unwrap_or_else(|_| "{}".to_string()),
    );

    if let Some(error) = trace_sink.last_error.as_deref() {
        eprintln!("TRACE_SINK_ERROR={error}");
    }

    let close_result = close_research_scorecard(&client, &base_url, &api_key, &card_id);

    match &close_result {
        Ok(scorecard) => {
            let bytes = serde_json::to_vec_pretty(scorecard).map_err(|error| error.to_string())?;

            std::fs::write(&scorecard_path, bytes).map_err(|error| error.to_string())?;

            println!("SCORECARD={scorecard_path}");

            println!(
                "FINAL_SCORECARD={}",
                serde_json::to_string(scorecard).unwrap_or_else(|_| "{}".to_string()),
            );
        }

        Err(error) => {
            eprintln!("SCORECARD_CLOSE_ERROR={error}");
        }
    }

    match episode {
        Ok(result) => {
            println!("TERMINATION={:?}", result.termination(),);

            println!("ATTEMPTS={}", result.decision_attempts(),);

            println!("EXECUTED={}", result.executed_steps(),);

            println!("ABSTENTIONS={}", result.abstentions(),);

            println!("FINAL_STATUS={:?}", result.final_status(),);
        }

        Err(error) => {
            return Err(format!("ARC episode failed: {error:?}"));
        }
    }

    close_result?;

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("ATHLESIA_REAL_ARC_FATAL={error}");

        std::process::exit(1);
    }
}
