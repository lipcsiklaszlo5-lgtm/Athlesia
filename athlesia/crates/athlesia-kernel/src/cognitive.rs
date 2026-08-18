
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

/// Kompetenciabecslés: mennyire ismeri a rendszer a feladatot,
/// mennyire bízik a hipotézisében, és mekkora keresési költséget jósol.
#[derive(Debug, Clone)]
pub struct CompetenceEstimate {
    pub familiarity: f32,
    pub structural_match: f32,
    pub hypothesis_confidence: f32,
    pub predicted_search_cost: f32,
    pub expected_information_gain: f32,
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
        known_programs: &[Program],
    ) -> CognitiveDecision {
        let _ = known_programs; // most nem használjuk, de a jövőben kell

        // Nagyon egyszerű becslés: ha van bármilyen kontextus-pontszám
        // a meta learnerben, akkor Solve, különben Explore.
        // Ez most egy placeholder, amit később finomítunk.
        let context_confidence = meta.priority_in_context(*features, 0); // 0 hipotézis id

        if context_confidence > 0.8 {
            CognitiveDecision::Solve
        } else if context_confidence > 0.5 {
            CognitiveDecision::Explore
        } else {
            CognitiveDecision::Guess
        }
    }

    /// Kompetenciabecslés kiszámítása.
    pub fn estimate(
        features: &FeatureVector,
        meta: &MetaLearner,
        input: &Grid,
        target: &Grid,
    ) -> CompetenceEstimate {
        // Konfidencia a 0. hipotézisre
        let conf = meta.priority_in_context(*features, 0) as f32;

        // Strukturális egyezés a Target Decomposer alapján
        let structural_match = if let Some(meta_grid) = TargetDecomposer::decompose_dimensions(input, target) {
            // Ha a dimenziók oszthatók, és van blokk-dekompozíció,
            // akkor erős strukturális egyezést feltételezünk.
            let block_count = (meta_grid.block_rows * meta_grid.block_cols) as f32;
            if block_count > 0.0 {
                1.0 / (1.0 + block_count) // minél több blokk, annál kisebb, de még mindig jel
            } else {
                0.0
            }
        } else {
            0.0
        };

        CompetenceEstimate {
            familiarity: conf,
            structural_match,
            hypothesis_confidence: conf,
            predicted_search_cost: 100.0,
            expected_information_gain: 0.5,
        }
    }
}
