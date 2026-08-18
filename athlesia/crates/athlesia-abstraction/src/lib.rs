
use std::collections::HashMap;
use athlesia_types::{Program, PrimName, Params};
use athlesia_knowledge::KnowledgeBase;

/// A Manhattan Kernel Abstraction Engine-je.
///
/// Ez a modul felelős a DSL fokozatos evolúciójáért:
/// - Mintakeresés anti-unifikációval
/// - MDL-pontozás
/// - Promóció a DSL-könyvtárba
///
/// A jelenlegi implementáció a leghosszabb közös részsorozatokat (LCS)
/// keresi a megoldott programokban, és ha elég gyakoriak, makróvá emeli.
pub struct AbstractionEngine;

impl AbstractionEngine {
    /// Megoldott programokból makrókat emel ki.
    ///
    /// A `solved_programs` a megoldott feladatok programjai.
    /// A `kb` a tudásbázis, amibe az új makrók kerülnek.
    /// A `threshold` a minimális előfordulási szám, ami felett egy minta
    /// érdemes a promócióra.
    pub fn extract_macros(
        solved_programs: &[Program],
        kb: &mut KnowledgeBase,
        threshold: usize,
    ) -> usize {
        let mut patterns: HashMap<Vec<(PrimName, Params)>, usize> = HashMap::new();

        // Gyűjtsük ki az összes lehetséges részsorozatot (1..max_len)
        let max_len = solved_programs.iter().map(|p| p.len()).max().unwrap_or(0);
        for len in 1..=max_len {
            for program in solved_programs {
                for window in program.windows(len) {
                    *patterns.entry(window.to_vec()).or_insert(0) += 1;
                }
            }
        }

        let mut added = 0;

        // Csak azok a minták érdekesek, amelyek legalább `threshold`-szor előfordulnak.
        for (pattern, count) in patterns.into_iter() {
            if count < threshold {
                continue;
            }

            // Ellenőrizzük, hogy nincs-e már ilyen makró.
            let exists = kb.get_all_macros().iter().any(|m| m.program == pattern);
            if exists {
                continue;
            }

            // MDL-pontozás: a leírási hossz csökkenése.
            // Itt egyszerű proxy: a minta hossza * előfordulás - a minta hossza.
            // Minél hosszabb és gyakoribb, annál nagyobb a nyereség.
            let gain = (pattern.len() as i64) * (count as i64 - 1) - 1;
            if gain < 0 {
                continue;
            }

            let name = format!("macro_{}", kb.get_all_macros().len());
            kb.add_macro(name, pattern);
            added += 1;
        }

        added
    }

    /// Két program anti-unifikációja.
    ///
    /// Visszaadja a leghosszabb közös részsorozatot (LCS), mint közös sémát.
    /// Ez a legegyszerűbb formája az anti-unifikációnak.
    pub fn anti_unify(a: &Program, b: &Program) -> Program {
        let mut dp = vec![vec![0usize; b.len() + 1]; a.len() + 1];

        for i in 1..=a.len() {
            for j in 1..=b.len() {
                if a[i - 1] == b[j - 1] {
                    dp[i][j] = dp[i - 1][j - 1] + 1;
                } else {
                    dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
                }
            }
        }

        // LCS visszafejtése
        let mut lcs = Vec::new();
        let (mut i, mut j) = (a.len(), b.len());
        while i > 0 && j > 0 {
            if a[i - 1] == b[j - 1] {
                lcs.push(a[i - 1].clone());
                i -= 1;
                j -= 1;
            } else if dp[i - 1][j] > dp[i][j - 1] {
                i -= 1;
            } else {
                j -= 1;
            }
        }
        lcs.reverse();
        lcs
    }
}
