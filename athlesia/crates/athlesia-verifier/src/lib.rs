
use athlesia_types::{Grid, Program, Budget};
use athlesia_executor::run_program;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationResult {
    Accept,
    Reject,
    Inconclusive,
}

#[derive(Debug, Clone, Default)]
pub struct Verifier;

impl Verifier {
    pub fn verify(&self, program: &Program, examples: &[(Grid, Grid)]) -> VerificationResult {
        if examples.is_empty() {
            return VerificationResult::Inconclusive;
        }

        for (input, expected) in examples {
            let mut budget = Budget { max_steps: 1000 };
            match run_program(program, input, &mut budget) {
                Ok(output) if output == *expected => continue,
                _ => return VerificationResult::Reject,
            }
        }

        VerificationResult::Accept
    }
}
