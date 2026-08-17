
use athlesia_types::{Grid, Program, PrimName, Params};
use athlesia_features::extract_features;
use athlesia_metalearner::MetaLearner;
use athlesia_verifier::{Verifier, VerificationResult};
use athlesia_synthesis::{synthesize, PrimitiveTemplate};

/// A Manhattan Kernel első tanuló magja.
/// Összeköti a jellemzőkinyerést, a MetaLearnert, a Verifiert és a Synthesis Engine-t.
#[derive(Debug, Default)]
pub struct CoreEngine {
    pub known_programs: Vec<Program>,
    pub meta: MetaLearner,
    pub verifier: Verifier,
}

impl CoreEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Megold egyetlen (input, target) párt. Ha a meglévő programok egyike sem működik,
    /// a Synthesis Engine megpróbál új programot generálni, verifikálja, és megtanulja.
    pub fn solve(&mut self, input: &Grid, target: &Grid) -> Option<Program> {
        let fv = extract_features(input);
        let ids: Vec<u64> = (0..self.known_programs.len() as u64).collect();

        // 1. Próbáljuk a már ismert programokat a MetaLearner rangsora szerint
        let ranked = self.meta.rank_in_context(fv, &ids);
        for id in ranked {
            let program = self.known_programs[id as usize].clone();
            let result = self.verifier.verify(&program, &[(input.clone(), target.clone())]);
            if result == VerificationResult::Accept {
                self.meta.record_success_in_context(fv, id);
                return Some(program);
            } else {
                self.meta.record_failure_in_context(fv, id);
            }
        }

        // 2. Ha nincs megfelelő, szintetizáljunk
        let templates = vec![
            PrimitiveTemplate::Translate,
            PrimitiveTemplate::ReflectH,
            PrimitiveTemplate::ReflectV,
            PrimitiveTemplate::Rotate90,
            PrimitiveTemplate::Recolor,
        ];

        if let Some(program) = synthesize(input, target, &templates) {
            // Verifikáljuk a szintetizált programot
            if self.verifier.verify(&program, &[(input.clone(), target.clone())]) == VerificationResult::Accept {
                let id = self.known_programs.len() as u64;
                self.known_programs.push(program.clone());
                self.meta.record_success_in_context(fv, id);
                return Some(program);
            }
        }

        None
    }
}
