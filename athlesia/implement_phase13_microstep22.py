#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. WorldModel: object_position_changed detektálása
p = pathlib.Path("crates/athlesia-world-model/src/lib.rs")
s = p.read_text()

old_obj_block = '''        // Objektum-szintű eltérések detektálása a perception szegmentációval.
        // Csak azonos dimenziójú grideken értelmes.
        if prediction.state.width == observation.state.width
            && prediction.state.height == observation.state.height
        {
            let pred_objects = segment(&prediction.state);
            let obs_objects = segment(&observation.state);
            if pred_objects.len() != obs_objects.len() {
                unexplained_features.push("object_count_changed".to_string());
            }
        }
'''

new_obj_block = '''        // Objektum-szintű eltérések detektálása a perception szegmentációval.
        // Csak azonos dimenziójú grideken értelmes.
        if prediction.state.width == observation.state.width
            && prediction.state.height == observation.state.height
        {
            let pred_objects = segment(&prediction.state);
            let obs_objects = segment(&observation.state);
            if pred_objects.len() != obs_objects.len() {
                unexplained_features.push("object_count_changed".to_string());
            } else if !pred_objects.is_empty() && !obs_objects.is_empty() {
                // Pozícióváltozás detektálása a centroidok összehasonlításával.
                let pred_centroid = centroid(&pred_objects[0]);
                let obs_centroid = centroid(&obs_objects[0]);
                if pred_centroid != obs_centroid {
                    unexplained_features.push("object_position_changed".to_string());
                }
            }
        }
'''

if old_obj_block not in s:
    print("[ERROR] Objektum-szintű blokk nem található.")
    sys.exit(1)
s = s.replace(old_obj_block, new_obj_block)

# centroid segédfüggvény hozzáadása a fájl végéhez
centroid_fn = r'''

/// Egyszerű centroid számítás egy GameObject sejtjeiből.
fn centroid(obj: &athlesia_perception::GameObject) -> (f64, f64) {
    if obj.cells.is_empty() {
        return (0.0, 0.0);
    }
    let sum_x: i64 = obj.cells.iter().map(|c| c.x as i64).sum();
    let sum_y: i64 = obj.cells.iter().map(|c| c.y as i64).sum();
    let n = obj.cells.len() as f64;
    (sum_x as f64 / n, sum_y as f64 / n)
}
'''
s = s.rstrip() + "\n" + centroid_fn
p.write_text(s)
print("[1] WorldModel frissítve: object_position_changed detektálás és centroid függvény.")

# 2. AbstractionEngine: object_position_changed priorizálása
p = pathlib.Path("crates/athlesia-abstraction/src/lib.rs")
s = p.read_text()

old_block = '''        // Ha van objektum-szintű változás, azt részesítsük előnyben a nyers
        // pixel_mismatch-sel szemben.
        let relation_pattern = if freq.contains_key("object_count_changed") {
            "object_count_change(A,B)".to_string()
        } else {
            freq.iter()
                .max_by_key(|(_, count)| *count)
                .map(|(feature, _)| feature.to_string())
                .unwrap_or_else(|| "pixel_mismatch".to_string())
        };

        let count = freq
            .get(relation_pattern.as_str())
            .copied()
            .unwrap_or(positive.len());
'''

new_block = '''        // Ha van objektum-szintű változás, azt részesítsük előnyben a nyers
        // pixel_mismatch-sel szemben.
        let relation_pattern = if freq.contains_key("object_position_changed") {
            "object_position_change(A,B)".to_string()
        } else if freq.contains_key("object_count_changed") {
            "object_count_change(A,B)".to_string()
        } else {
            freq.iter()
                .max_by_key(|(_, count)| *count)
                .map(|(feature, _)| feature.to_string())
                .unwrap_or_else(|| "pixel_mismatch".to_string())
        };

        let count = freq
            .get(relation_pattern.as_str())
            .copied()
            .unwrap_or(positive.len());

        // Ha a meghatározó jellemző objektumpozíció-változás,
        // akkor a confidence legyen 1.0, mert a reláció egyértelmű.
        let confidence_override = relation_pattern == "object_position_change(A,B)";
'''

# Az avg_mismatch számítás előtt beszúrjuk az override-ot
old_conf = '''        // Átlagos mismatch_score mint kezdeti confidence.
        let avg_mismatch = positive
            .iter()
            .map(|r| r.mismatch_score)
            .sum::<f64>()
            / positive.len() as f64;
'''
new_conf = '''        // Átlagos mismatch_score mint kezdeti confidence.
        let avg_mismatch = positive
            .iter()
            .map(|r| r.mismatch_score)
            .sum::<f64>()
            / positive.len() as f64;

        let confidence = if confidence_override { 1.0 } else { avg_mismatch.min(1.0) };
'''

if old_block not in s:
    print("[ERROR] A relation_pattern blokk nem található.")
    sys.exit(1)
s = s.replace(old_block, new_block)

if old_conf not in s:
    print("[ERROR] Az avg_mismatch blokk nem található.")
    sys.exit(1)
s = s.replace(old_conf, new_conf)

# A CandidateConcept confidence mezőjét frissítjük
old_candidate = '''        Some(athlesia_hypothesis::CandidateConcept {
            sketch,
            evidence,
            confidence: avg_mismatch.min(1.0),
        })
'''
new_candidate = '''        Some(athlesia_hypothesis::CandidateConcept {
            sketch,
            evidence,
            confidence,
        })
'''
if old_candidate not in s:
    print("[ERROR] CandidateConcept blokk nem található.")
    sys.exit(1)
s = s.replace(old_candidate, new_candidate)

p.write_text(s)
print("[2] AbstractionEngine frissítve: object_position_changed prioritás és confidence override.")

# 3. Valós interaktív környezeti teszt
p = pathlib.Path("crates/athlesia-core/tests/openworld_experiment_cycle_test.rs")
s = p.read_text()

old_closure = '''    let mut executed = false;
    let outcome = OpenWorldCycle::run_experiment_cycle(
        &wm,
        &mut kb,
        &mut meta,
        request,
        |_| {
            executed = true;
            // Szimulált megfigyelés dimenzióeltéréssel, hogy a ciklus
            // Verified kimenetet adjon. Ez a teszt a generikus ciklus
            // kontrollfolyamát ellenőrzi, nem a valódi környezetet.
            Observation {
                state: Grid::new(3, 3),
            }
        },
    );
'''
new_closure = '''    let mut executed = false;
    let outcome = OpenWorldCycle::run_experiment_cycle(
        &wm,
        &mut kb,
        &mut meta,
        request,
        |_| {
            executed = true;
            // Valós interaktív környezet: a C akció jobbra mozdítja az objektumot.
            let observed_grid = env.step(&ProbeAction::C);
            Observation { state: observed_grid }
        },
    );
'''
if old_closure not in s:
    print("[ERROR] A closure blokk nem található.")
    sys.exit(1)
s = s.replace(old_closure, new_closure)
p.write_text(s)
print("[3] Teszt frissítve: valós környezeti megfigyelés használata.")

# 4. Core tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-core"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Core tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Core tesztek zöldek.")

# 5. Teljes workspace teszt
result = subprocess.run(
    ["cargo", "test", "--workspace", "--no-fail-fast"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Teljes workspace tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Teljes workspace tesztek zöldek.")

# 6. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 22: detect object_position_changed and use real interactive environment in cycle test"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
