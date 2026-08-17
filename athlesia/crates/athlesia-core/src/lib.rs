
use athlesia_types::{Grid, Program};
use athlesia_features::extract_features;
use athlesia_metalearner::MetaLearner;
use athlesia_verifier::{Verifier, VerificationResult};
use athlesia_synthesis::{synthesize, PrimitiveTemplate};
use athlesia_search::search;

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
    /// Megoldja a feladatot, és visszaadja a megtalált programot.
    pub fn solve(&mut self, input: &Grid, target: &Grid) -> Option<Program> {
        self.solve_with_steps(input, target).0
    }

    /// Megoldja a feladatot, és visszaadja a megtalált programot,
    /// valamint azt, hogy hány hipotézist próbált ki (keresési lépések).
    pub fn solve_with_steps(&mut self, input: &Grid, target: &Grid) -> (Option<Program>, usize) {
        let fv = extract_features(input);
        let ids: Vec<u64> = (0..self.known_programs.len() as u64).collect();

        let mut steps = 0;

        // 1. Próbáljuk a már ismert programokat a MetaLearner rangsora szerint
        let ranked = self.meta.rank_in_context(fv, &ids);
        for id in ranked {
            steps += 1;
            let program = self.known_programs[id as usize].clone();
            let result = self.verifier.verify(&program, &[(input.clone(), target.clone())]);
            if result == VerificationResult::Accept {
                self.meta.record_success_in_context(fv, id);
                return (Some(program), steps);
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
            steps += 1; // a szintézis egy próbálkozásnak számít
            // Verifikáljuk a szintetizált programot
            if self.verifier.verify(&program, &[(input.clone(), target.clone())]) == VerificationResult::Accept {
                let id = self.known_programs.len() as u64;
                self.known_programs.push(program.clone());
                self.meta.record_success_in_context(fv, id);
                return (Some(program), steps);
            }
        }

        // 3. Ha a szintézis nem járt sikerrel, próbáljuk a többlépéses keresést
        if let Some(program) = search(input, target, 3) {
            steps += 1;
            if self.verifier.verify(&program, &[(input.clone(), target.clone())]) == VerificationResult::Accept {
                let id = self.known_programs.len() as u64;
                self.known_programs.push(program.clone());
                self.meta.record_success_in_context(fv, id);
                return (Some(program), steps);
            }
        }

        (None, steps)
    }
}
