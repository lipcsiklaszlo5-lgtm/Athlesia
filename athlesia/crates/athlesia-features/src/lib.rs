
use athlesia_types::{Grid, GRID_SIZE};
use athlesia_perception::{segment, touches};

/// FeatureVector: ezeket a jellemzőket fogja később a MetaLearner használni.
/// A Hash miatt könnyen lehet HashMap kulcs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeatureVector {
    pub object_count: u8,
    pub color_counts: [u8; 4],
    pub touching_pairs: u8,
}

pub fn extract_features(grid: &Grid) -> FeatureVector {
    let objects = segment(grid);
    let object_count = objects.len() as u8;

    // Szín gyakoriságok
    let mut color_counts = [0u8; 4];
    for row in &grid.cells {
        for &cell in row {
            if cell < 4 {
                color_counts[cell as usize] += 1;
            }
        }
    }

    // Érintkező párok száma
    let mut touching_pairs = 0;
    for i in 0..objects.len() {
        for j in (i + 1)..objects.len() {
            if touches(&objects[i], &objects[j]) {
                touching_pairs += 1;
            }
        }
    }

    FeatureVector {
        object_count,
        color_counts,
        touching_pairs,
    }
}
