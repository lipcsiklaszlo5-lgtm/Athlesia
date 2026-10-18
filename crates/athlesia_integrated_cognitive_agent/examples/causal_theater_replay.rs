//! Reproduce the browser's live opaque challenge with the same kernel and seed.
//! Stdout is original Rust JSONL. Diagnostics and replay rejection go to stderr.
use athlesia_integrated_cognitive_agent::opaque_action_port::{
    OpaqueInteractionSession, OpaqueSessionPhase,
};

fn replay(seed: u32) -> Result<String, String> {
    let mut session = OpaqueInteractionSession::new(u64::from(seed))?;
    let token = session.action_token().to_owned();
    session.send_action(&token)?;
    session.sabotage()?;
    for _ in 0..8 {
        if session.phase() == OpaqueSessionPhase::Revised {
            break;
        }
        session.probe()?;
    }
    if session.phase() != OpaqueSessionPhase::Revised {
        return Err("ABSTAIN: bounded probes did not produce a unique revision".to_owned());
    }
    session.verify_holdout()?;
    if session.phase() != OpaqueSessionPhase::HoldoutVerified {
        return Err("ABSTAIN: both independent holdouts must pass".to_owned());
    }
    let jsonl = session.observation_jsonl();
    let summary = jsonl
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|record| record["record"] == "summary")
        .last()
        .ok_or_else(|| "missing Rust summary".to_owned())?;
    for field in [
        "holdout_positive",
        "holdout_negative",
        "ledger_verified",
        "source_model_unchanged",
        "first_attempt_transfer",
    ] {
        if summary[field] != true {
            return Err(format!("unverified Rust result: {field}"));
        }
    }
    let rejection = session
        .verify_holdout()
        .err()
        .ok_or_else(|| "completed holdout unexpectedly accepted a replay".to_owned())?;
    if session.observation_jsonl() != jsonl {
        return Err("rejected replay changed the original Rust trace".to_owned());
    }
    eprintln!("Rust session replay rejected: {rejection}");
    Ok(jsonl)
}

fn main() {
    let input = std::env::args().nth(1).unwrap_or_else(|| "42".to_owned());
    let result = input
        .parse::<u32>()
        .map_err(|_| "seed must be an unsigned 32-bit integer".to_owned())
        .and_then(replay);
    match result {
        Ok(jsonl) => print!("{jsonl}"),
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_challenges_replay_deterministically() {
        for seed in [0, 1, 7, 8, 42, 77, 101, u32::MAX] {
            let first = replay(seed).expect("bounded native challenge");
            let second = replay(seed).expect("same seed must replay");
            assert_eq!(first, second, "seed={seed}");
            assert!(first.contains("\"holdout_positive\":true"));
            assert!(first.contains("\"holdout_negative\":true"));
        }
    }

    #[test]
    fn user_cannot_skip_the_native_phase_boundaries() {
        let mut session = OpaqueInteractionSession::new(42).unwrap();
        let before = session.observation_jsonl();
        assert!(session.verify_holdout().is_err());
        assert!(session.sabotage().is_err());
        assert!(session.send_action("user-supplied-wrong-token").is_err());
        assert_eq!(session.observation_jsonl(), before);
    }
}
