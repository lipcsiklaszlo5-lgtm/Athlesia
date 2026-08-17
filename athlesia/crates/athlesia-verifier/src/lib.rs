
use athlesia_types::{Grid, Program, Budget};
use athlesia_executor::run_program;

/// Elfogadási/elutasítási eredmény.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationResult {
    Accept,
    Reject,
    Inconclusive,
}

/// Bizonyíték: múltbeli megfigyelések (bemenet-kimenet párok).
pub type Evidence = Vec<(Grid, Grid)>;

/// A Manhattan Kernel Verifier modulja.
#[derive(Debug, Clone, Default)]
pub struct Verifier {
    // Ha később kell állapotot tartani, itt lehet.
}

impl Verifier {
    pub fn new() -> Self {
        Self::default()
    }

    /// Programhelyesség-ellenőrzés tanulópéldák ellen.
    pub fn verify(&self, program: &Program, examples: &Evidence) -> VerificationResult {
        if examples.is_empty() {
            return VerificationResult::Inconclusive;
        }

        for (input, expected) in examples {
            let mut budget = Budget { max_steps: 1000, max_depth: 100 };
            match run_program(program, input, &mut budget) {
                Ok(output) if output == *expected => continue,
                _ => return VerificationResult::Reject,
            }
        }

        VerificationResult::Accept
    }

    /// Hipotézis-ellenőrzés a teljes megfigyelt előzmény ellen.
    /// Ugyanaz, mint a `verify`, de kifejezettebb nevet adunk neki,
    /// hogy a hívó szándéka világos legyen.
    pub fn verify_hypothesis(&self, hypothesis: &Program, history: &Evidence) -> VerificationResult {
        self.verify(hypothesis, history)
    }

    /// Két program szemantikai ekvivalenciája a megadott példákon.
    /// Csak akkor `Accept`, ha minden példán ugyanazt a kimenetet adják.
    pub fn verify_equivalence(&self, program_a: &Program, program_b: &Program, examples: &Evidence) -> VerificationResult {
        if examples.is_empty() {
            return VerificationResult::Inconclusive;
        }

        for (input, _) in examples {
            let mut budget_a = Budget { max_steps: 1000, max_depth: 100 };
            let mut budget_b = Budget { max_steps: 1000, max_depth: 100 };

            let output_a = run_program(program_a, input, &mut budget_a);
            let output_b = run_program(program_b, input, &mut budget_b);

            match (output_a, output_b) {
                (Ok(a), Ok(b)) if a == b => continue,
                _ => return VerificationResult::Reject,
            }
        }

        VerificationResult::Accept
    }
}
