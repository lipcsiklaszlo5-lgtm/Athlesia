#!/usr/bin/env python3
import os, subprocess, sys, pathlib

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Abstraction Engine lib.rs teljes újraírása a dokumentum 5. fejezete szerint
write_file("crates/athlesia-abstraction/src/lib.rs", r'''
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
            if gain <= 0 {
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
''')
print("[1] Abstraction Engine lib.rs teljesen újraírva.")

# 2. Tesztek teljes újraírása
write_file("crates/athlesia-abstraction/tests/abstraction_full_test.rs", r'''
use athlesia_abstraction::AbstractionEngine;
use athlesia_knowledge::KnowledgeBase;
use athlesia_types::{PrimName, Params, Program};

#[test]
fn extracts_frequent_subsequence_as_macro() {
    let mut kb = KnowledgeBase::new();
    let engine = AbstractionEngine;

    let translate: (PrimName, Params) = (PrimName::Translate, Params::Translate(1, 0));
    let recolor: (PrimName, Params) = (PrimName::Recolor, Params::Recolor([
        Color(1), Color(0), Color(2), Color(3), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)
    ]));

    // Gyakori részsorozat: translate + recolor
    let pattern: Program = vec![translate, recolor];

    let solved = vec![
        pattern.clone(),
        vec![translate, recolor, (PrimName::ReflectH, Params::None)],
        vec![(PrimName::Rotate90, Params::None), translate, recolor],
        pattern.clone(),
        vec![translate, recolor, (PrimName::ReflectV, Params::None)],
    ];

    let added = engine.extract_macros(&solved, &mut kb, 4);
    assert!(added >= 1, "Legalább egy makrót hozzá kell adni");

    let has_pattern = kb.get_all_macros().iter().any(|m| m.program == pattern);
    assert!(has_pattern, "A gyakori részsorozatot makróként kell tárolni");
}

#[test]
fn anti_unify_finds_common_subsequence() {
    let a: Program = vec![
        (PrimName::Translate, Params::Translate(1, 0)),
        (PrimName::Rotate90, Params::None),
        (PrimName::Recolor, Params::Recolor([
            Color(1), Color(0), Color(2), Color(3), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)
        ])),
    ];
    let b: Program = vec![
        (PrimName::Rotate90, Params::None),
        (PrimName::Translate, Params::Translate(1, 0)),
        (PrimName::Recolor, Params::Recolor([
            Color(1), Color(0), Color(2), Color(3), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)
        ])),
    ];

    let lcs = AbstractionEngine::anti_unify(&a, &b);
    // A leghosszabb közös részsorozat: [Translate, Recolor] vagy [Rotate90, Recolor]
    assert!(lcs.len() == 2);
    assert_eq!(lcs[1], a[2]);
}

#[test]
fn does_not_extract_infrequent_patterns() {
    let mut kb = KnowledgeBase::new();
    let engine = AbstractionEngine;

    let a: Program = vec![(PrimName::ReflectH, Params::None)];
    let b: Program = vec![(PrimName::ReflectV, Params::None)];

    let solved = vec![a, b];

    let added = engine.extract_macros(&solved, &mut kb, 2);
    assert_eq!(added, 0);
}
''')
print("[2] Abstraction Engine tesztek hozzáadva.")

# 3. Tesztek futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-abstraction", "--test", "abstraction_full_test"], capture_output=True, text=True, check=False)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Abstraction Engine tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Abstraction Engine tesztek zöldek.")

# 4. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Finalize Abstraction Engine with anti-unification and MDL scoring"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
