#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. CoreEngine módosítása: meglévő programok kompozíciójának kipróbálása
p = pathlib.Path("crates/athlesia-core/src/lib.rs")
s = p.read_text()

# A régi "3. Ha a szintézis nem járt sikerrel" blokk elé szúrjuk be a kompozíciós lépést
old_search_block = '''        // 3. Ha a szintézis nem járt sikerrel, próbáljuk a többlépéses keresést
        let max_score = (target.width as usize * target.height as usize) as f32;
        let mut telemetry = SearchTelemetry::new(max_score);'''

new_composition_block = '''        // 2.5 A meglévő programok kompozícióinak kipróbálása (kettő hosszig)
        let known = self.known_programs.clone();
        for p1 in &known {
            for p2 in &known {
                let mut combined = p1.clone();
                combined.extend(p2.clone());
                steps += 1;
                if self.verifier.verify(&combined, &vec![(input.clone(), target.clone())]) == VerificationResult::Accept {
                    let id = self.known_programs.len() as u64;
                    self.known_programs.push(combined.clone());
                    self.meta.record_success_in_context(fv, id);
                    return (Some(combined), steps);
                }
            }
        }

        // 3. Ha a szintézis nem járt sikerrel, próbáljuk a többlépéses keresést
        let max_score = (target.width as usize * target.height as usize) as f32;
        let mut telemetry = SearchTelemetry::new(max_score);'''

if old_search_block not in s:
    print("[ERROR] A keresési blokk nem található a CoreEngine-ben.")
    sys.exit(1)

s = s.replace(old_search_block, new_composition_block)
write_file(p, s)
print("[1] CoreEngine frissítve: meglévő programok kompozíciójának keresése.")

# 2. Új teszt a koncepciótranszferre
test_code = r'''
use athlesia_core::CoreEngine;
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
fn concept_transfer_composes_known_programs() {
    let mut engine = CoreEngine::new();

    // Tanuljuk meg a ReflectH-t
    let input_refl = grid_from_rows(vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
    ]);
    let target_refl = grid_from_rows(vec![
        vec![3, 2, 1],
        vec![6, 5, 4],
        vec![9, 8, 7],
    ]);
    let (_, steps_refl) = engine.solve_with_steps(&input_refl, &target_refl);
    assert!(steps_refl > 0, "ReflectH tanulásnak kellett történnie");

    // Tanuljuk meg a Translate(1,0)-t
    let input_trans = grid_from_rows(vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
    ]);
    let target_trans = grid_from_rows(vec![
        vec![0, 1, 2],
        vec![0, 4, 5],
        vec![0, 7, 8],
    ]);
    let (_, steps_trans) = engine.solve_with_steps(&input_trans, &target_trans);
    assert!(steps_trans > 0, "Translate tanulásnak kellett történnie");

    // Most jön a kombinált feladat: ReflectH + Translate(1,0)
    let input_comb = grid_from_rows(vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
    ]);
    let target_comb = grid_from_rows(vec![
        vec![0, 3, 2],
        vec![0, 6, 5],
        vec![0, 9, 8],
    ]);

    let (result, steps_comb) = engine.solve_with_steps(&input_comb, &target_comb);
    assert!(result.is_some(), "A kombinált feladatot meg kellett oldani");
    // A kompozíciós lépésnek gyorsnak kell lennie, mert a programok már ismertek.
    assert!(steps_comb < steps_refl + steps_trans, "A kompozíciónak kevesebb lépéssel kell megoldódnia");
}
'''
write_file("crates/athlesia-core/tests/concept_transfer_test.rs", test_code)
print("[2] concept_transfer_test.rs létrehozva.")

# 3. Core tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-core"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Phase 10 tesztek nem mentek át.")
    sys.exit(1)

print("\n[SUCCESS] Phase 10 tesztek zöldek.")

# 4. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 10: concept transfer via composition of known programs in CoreEngine"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
