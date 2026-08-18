#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. cognitive.rs frissítése a pontosabb structural_match számítással
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
    /// Döntési logika:
    /// - Magas konfidencia -> Solve
    /// - Közepes konfidencia -> Explore
    /// - Van valamilyen ismert program, és a keresés olcsónak ígérkezik -> Guess
    /// - Különben -> Abstain
    pub fn decide(
        features: &FeatureVector,
        meta: &MetaLearner,
        known_programs: &[Program],
        input: &Grid,
        target: &Grid,
    ) -> CognitiveDecision {
        let estimate = Self::estimate(features, meta, input, target);

        // A prediktált keresési költséget csökkentjük, ha van ismert program,
        // mert van miből kiindulni.
        let mut predicted_search_cost = estimate.predicted_search_cost;
        if !known_programs.is_empty() {
            predicted_search_cost = predicted_search_cost.min(10.0);
        }

        if estimate.hypothesis_confidence > 0.8 {
            CognitiveDecision::Solve
        } else if estimate.hypothesis_confidence > 0.5 {
            CognitiveDecision::Explore
        } else if predicted_search_cost < 50.0 {
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

        // Strukturális egyezés a Target Decomposer alapján.
        // Csak akkor van strukturális jel, ha a célrács az input méretének
        // többszöröse (legalább 2x2 blokk), és a blokkok nagy része
        // felismerhető transzformált input.
        let mut structural_match = 0.0;
        if let Some(decomp) = TargetDecomposer::decompose_dimensions(input, target) {
            let block_count = decomp.block_rows * decomp.block_cols;
            if block_count > 1 {
                if let Some(meta_grid) = TargetDecomposer.decompose(input, target) {
                    let total = meta_grid.rows * meta_grid.cols;
                    let matched = meta_grid.cells.iter().filter(|c| c.is_some()).count();
                    if total > 0 {
                        structural_match = matched as f32 / total as f32;
                    }
                }
            }
        }

        // Egyszerűségi pont: ha a struktúra teljesen felismert, akkor a
        // szabály nagy valószínűséggel egyszerű, ezért magas pontot adunk.
        let simplicity_score = if structural_match > 0.0 {
            structural_match
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
print("[1] cognitive.rs frissítve: structural_match most a MetaGrid felismerésén alapul.")

# 2. Új teszt a strukturális elemzésre
test_code = r'''
use athlesia_kernel::cognitive::{CognitiveController, CompetenceEstimate};
use athlesia_features::FeatureVector;
use athlesia_metalearner::MetaLearner;
use athlesia_types::{Grid, Color};

fn grid_from_rows(rows: Vec<Vec<u8>>) -> Grid {
    let height = rows.len() as u8;
    let width = if height > 0 { rows[0].len() as u8 } else { 0 };
    let mut cells = Vec::new();
    for row in &rows {
        for &c in row {
            cells.push(Color(c));
        }
    }
    Grid { width, height, cells }
}

#[test]
fn structural_match_high_for_repeated_identity_blocks() {
    let input = grid_from_rows(vec![
        vec![1, 2],
        vec![3, 4],
    ]);
    let target = grid_from_rows(vec![
        vec![1,2,1,2],
        vec![3,4,3,4],
        vec![1,2,1,2],
        vec![3,4,3,4],
    ]);

    let meta = MetaLearner::new();
    let fv = FeatureVector::default();
    let estimate = CognitiveController::estimate(&fv, &meta, &input, &target);
    assert!(estimate.structural_match > 0.9, "Az identitás ismétlődésnek magas strukturális egyezést kell adnia, de {} volt", estimate.structural_match);
}

#[test]
fn structural_match_zero_for_trivial_same_size() {
    let input = grid_from_rows(vec![vec![1,2], vec![3,4]]);
    let target = grid_from_rows(vec![vec![1,2], vec![3,4]]);

    let meta = MetaLearner::new();
    let fv = FeatureVector::default();
    let estimate = CognitiveController::estimate(&fv, &meta, &input, &target);
    assert_eq!(estimate.structural_match, 0.0, "Azonos méretű grid nem ad strukturális jelet");
}
'''

write_file("crates/athlesia-kernel/tests/structural_analysis_test.rs", test_code)
print("[2] structural_analysis_test.rs létrehozva.")

# 3. Tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-kernel"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] A kernel tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Kernel tesztek zöldek.")

# 4. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 2: structural_match now based on MetaGrid recognition ratio"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
