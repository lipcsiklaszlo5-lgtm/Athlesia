use std::env;

use athlesia_integrated_cognitive_agent::fresh_world_challenge;

const DEFAULT_SEED: u64 = 0xA7_1E_5E_ED;

fn main() {
    let seed = env::args()
        .nth(1)
        .map(|value| value.parse::<u64>().expect("seed must be an unsigned integer"))
        .unwrap_or(DEFAULT_SEED);

    match fresh_world_challenge::run(seed) {
        Ok(report) => print!("{}", report.jsonl()),
        Err(error) => {
            eprintln!("fresh world challenge failed: {error}");
            std::process::exit(1);
        }
    }
}
