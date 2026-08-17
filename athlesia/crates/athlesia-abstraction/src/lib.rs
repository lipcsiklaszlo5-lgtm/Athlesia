
use std::collections::HashMap;
use athlesia_types::Program;
use athlesia_knowledge::KnowledgeBase;

/// Abstraction Engine: gyakori programminták kinyerése és makrósítása.
///
/// Az aktuális implementáció a legegyszerűbb, de valódi absztrakció:
/// megkeresi azokat az egylépéses programokat (primitív + paraméter),
/// amelyek legalább `threshold` alkalommal előfordulnak a megoldott
/// programok között, és amelyek még nincsenek makróként a tudásbázisban.
/// A megtalált mintákat makróként hozzáadja a tudásbázishoz.
///
/// A későbbi fázisokban ez bővül majd anti-unifikációval és MDL-pontozással.
pub struct AbstractionEngine;

impl AbstractionEngine {
    /// Megoldott programokból makrókat emel ki.
    /// `solved_programs`: a megoldott feladatok programjai.
    /// `kb`: a tudásbázis, amibe az új makrók kerülnek.
    /// `threshold`: hány előfordulás felett tekintünk egy mintát érdemesnek.
    pub fn extract_macros(
        solved_programs: &[Program],
        kb: &mut KnowledgeBase,
        threshold: usize,
    ) -> usize {
        let mut counts: HashMap<Program, usize> = HashMap::new();

        for program in solved_programs {
            // Egylépéses minták
            for step in program {
                let sub = vec![step.clone()];
                *counts.entry(sub).or_insert(0) += 1;
            }

            // Kétlépéses összefüggő minták
            if program.len() >= 2 {
                for window in program.windows(2) {
                    let sub: Program = window.to_vec();
                    *counts.entry(sub).or_insert(0) += 1;
                }
            }
        }

        let mut added = 0;
        for (pattern, count) in counts {
            if count >= threshold {
                // Ellenőrizzük, hogy nincs-e már ilyen program a könyvtárban
                let exists = kb.get_all_macros().iter().any(|m| m.program == pattern);
                if !exists {
                    let name = format!("macro_{}", kb.get_all_macros().len());
                    kb.add_macro(name, pattern);
                    added += 1;
                }
            }
        }

        added
    }
}
