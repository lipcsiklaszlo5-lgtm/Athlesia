#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. cognitive.rs újratöltése, javítás a structural_match számításában
cognitive_rs = r'''
use athlesia_features::FeatureVector;
use athlesia_metalearner::MetaLearner;
use athlesia_structure::TargetDecomposer;
use athlesia_types::{Program, Grid};

/// Döntés, amit a kognitív kontroller hozhat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CognitiveDecision {
    Solve,
    Explore,
    Guess,
    Abstain,
}

/// Kemény kényszerek: ezek megsértése esetén a hipotézis érvénytelen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardConstraint {
    GridBounds,
    ValidColor,
    ValidAction,
    ValidDimension,
}

/// Lágy priorok: valószínűségi súlyt adnak, de felülírhatók.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoftPrior {
    SimpleTransformation,
    ObjectPersistence,
    Symmetry,
    Locality,
    SameRuleRepeated,
    FewExceptions,
    Compositionality,
}

/// Kompetenciabecslés: mennyire ismeri a rendszer a feladatot,
/// mennyire bízik a hipotézisében, és mekkora keresési költséget jósol.
#[derive(Debug, Clone)]
pub struct CompetenceEstimate {
    pub familiarity: f32,
    pub structural_match: f32,
    pub hypothesis_confidence: f32,
    pub predicted_search_cost: f32,
    pub expected_information_gain: f32,
    pub simplicity_score: f32,
}

/// Kognitív kontroller: dönt a cselekvésről a jellemzővektor és a
/// MetaLearner állapota alapján.
pub struct CognitiveController;

impl CognitiveController {
    /// Egyszerű döntési logika:
    /// - ha van hasonló kontextusú ismert program, és a konfidencia magas -> Solve
    /// - ha a hasonlóság közepes -> Explore
    /// - ha nincs semmi, de a keresés olcsónak ígérkezik -> Guess
    /// - egyébként -> Abstain
    pub fn decide(
        features: &FeatureVector,
        meta: &MetaLearner,
        _known_programs: &[Program],
        input: &Grid,
        target: &Grid,
    ) -> CognitiveDecision {
        let estimate = Self::estimate(features, meta, input, target);

        // A döntéshez használjuk a becslést.
        if estimate.hypothesis_confidence > 0.8 && estimate.structural_match > 0.5 {
            CognitiveDecision::Solve
        } else if estimate.hypothesis_confidence > 0.5 || estimate.structural_match > 0.3 {
            CognitiveDecision::Explore
        } else if estimate.predicted_search_cost < 50.0 {
            CognitiveDecision::Guess
        } else {
            CognitiveDecision::Abstain
        }
    }

    /// Kompetenciabecslés kiszámítása.
    pub fn estimate(
        features: &FeatureVector,
        meta: &MetaLearner,
        input: &Grid,
        target: &Grid,
    ) -> CompetenceEstimate {
        // Konfidencia a 0. hipotézisre (placeholder, később finomítjuk)
        let conf = meta.priority_in_context(*features, 0) as f32;

        // Strukturális egyezés a Target Decomposer alapján
        let structural_match = if let Some(meta_grid) = TargetDecomposer::decompose_dimensions(input, target) {
            let block_count = (meta_grid.block_rows * meta_grid.block_cols) as f32;
            // A triviális 1 blokkos dekompozíció nem ad strukturális jelet.
            if block_count > 1.0 {
                1.0 / (1.0 + block_count)
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Egyszerűségi pont: kevés blokk magas pontot ad,
        // sok blokk alacsonyat.
        let simplicity_score = if structural_match > 0.0 {
            // A blokkok számából számítjuk, de most csak közelítés
            1.0 - structural_match
        } else {
            0.0
        };

        // A prediktált keresési költség becslése (most egyszerű: minél alacsonyabb
        // a konfidencia, annál drágább a keresés).
        let predicted_search_cost = 100.0 * (1.0 - conf);

        // Az elvárt információnyerés: bizonytalanság esetén magasabb.
        let expected_information_gain = 1.0 - conf;

        CompetenceEstimate {
            familiarity: conf,
            structural_match,
            hypothesis_confidence: conf,
            predicted_search_cost,
            expected_information_gain,
            simplicity_score,
        }
    }
}
'''

write_file("crates/athlesia-kernel/src/cognitive.rs", cognitive_rs)
print("[1] cognitive.rs frissítve (structural_match triviális kizárással).")

# 2. Teszt változatlan, de most már át kell mennie
# (a prior_test.rs már létezik, nem írjuk felül)

# 3. Tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-kernel", "--test", "prior_test"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] A prior tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Prior tesztek zöldek.")

# 4. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Fix structural_match to ignore trivial 1-block decomposition"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
