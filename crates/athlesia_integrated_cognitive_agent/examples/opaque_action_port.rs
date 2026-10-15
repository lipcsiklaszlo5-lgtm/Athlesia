//! Execute a JSON challenge through the opaque action-port boundary.
//!
//! Usage:
//!   cargo run --example opaque_action_port -- path/to/challenge.json

use std::{env, fs, process::ExitCode};

use athlesia_integrated_cognitive_agent::{
    custom_mechanism::CustomChallengeSpec,
    opaque_action_port::run_opaque_custom,
};

fn main() -> ExitCode {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: opaque_action_port <challenge.json>");
        return ExitCode::from(2);
    };
    let input = match fs::read_to_string(&path) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("cannot read {path}: {error}");
            return ExitCode::from(2);
        }
    };
    let spec = match CustomChallengeSpec::from_json(&input) {
        Ok(spec) => spec,
        Err(error) => {
            eprintln!("invalid challenge: {error}");
            return ExitCode::from(2);
        }
    };
    let report = match run_opaque_custom(spec) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("opaque challenge rejected: {error}");
            return ExitCode::from(1);
        }
    };
    print!("{}", report.jsonl());
    if report.succeeded() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
