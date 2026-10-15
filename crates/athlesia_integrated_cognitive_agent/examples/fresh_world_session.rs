use std::env;

use athlesia_integrated_cognitive_agent::fresh_world_session::FreshWorldSession;

const DEFAULT_SEED: u64 = 0xA7_1E_5E_ED;

fn main() {
    let args: Vec<String> = env::args().collect();
    let seed = args
        .get(1)
        .map(|value| value.parse::<u64>().expect("seed must be an unsigned integer"))
        .unwrap_or(DEFAULT_SEED);
    let run_sabotage = args.iter().any(|arg| arg == "--sabotage" || arg == "--transfer");
    let run_transfer = args.iter().any(|arg| arg == "--transfer");

    let mut session = FreshWorldSession::new(seed);
    print!("{}", session.world_jsonl());
    if run_sabotage {
        match session.sabotage() {
            Ok(jsonl) => print!("{}", jsonl),
            Err(error) => {
                eprintln!("sabotage failed: {error}");
                std::process::exit(1);
            }
        }
    }
    if run_transfer {
        match session.transfer_to_unknown() {
            Ok(jsonl) => print!("{}", jsonl),
            Err(error) => {
                eprintln!("transfer failed: {error}");
                std::process::exit(1);
            }
        }
    }
}
