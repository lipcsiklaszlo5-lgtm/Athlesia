#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Planner lib.rs bővítése
p = pathlib.Path("crates/athlesia-planner/src/lib.rs")
s = p.read_text()

# ActionValue struktúra beszúrása a PlannerMode után
action_value_def = '''
/// Egy akció értékelése a döntési ciklushoz.
/// A mezők súlyozva kombinálhatók a `value` függvényben.
#[derive(Debug, Clone)]
pub struct ActionValue {
    pub expected_information_gain: f32,
    pub expected_progress: f32,
    pub action_cost: f32,
    pub risk: f32,
}

'''
# Beszúrjuk a PlannerMode definíció után
marker = "/// A tervező üzemmódja.\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum PlannerMode {\n    GoalDirected,\n    Exploration,\n}\n"
if marker not in s:
    print("[ERROR] PlannerMode enum nem található.")
    sys.exit(1)
s = s.replace(marker, marker + action_value_def)

# Új impl blokk hozzáfűzése a fájl végéhez
new_impl = r'''

impl Planner {
    /// Kiszámítja egy akció ActionValue értékelését.
    ///
    /// - `expected_information_gain`: a predikció bizonytalansága (1 - confidence).
    /// - `expected_progress`: ha van cél, a pixel-egyezés javulása az akció után.
    /// - `action_cost`: egyszerűsített költség (jelenleg konstans 1).
    /// - `risk`: jelenleg 0, később bővíthető.
    pub fn compute_action_value(
        &self,
        current: &Grid,
        target: Option<&Grid>,
        action: &Action,
        wm: &WorldModel,
    ) -> ActionValue {
        let query = Query {
            state: current.clone(),
            action: action.clone(),
        };
        let prediction = wm.predict(&query.state, &query.action);
        let uncertainty = 1.0 - prediction.confidence as f32;

        let info_gain = uncertainty;

        let progress = if let Some(target_grid) = target {
            let before = pixel_match(current, target_grid);
            let after = pixel_match(&prediction.state, target_grid);
            (after as f32 - before as f32) // lehet negatív is
        } else {
            0.0
        };

        ActionValue {
            expected_information_gain: info_gain,
            expected_progress: progress,
            action_cost: 1.0,
            risk: 0.0,
        }
    }

    /// Kiválasztja a legjobb akciót a megadott súlyokkal.
    ///
    /// `value = α * info_gain + β * progress - γ * cost - δ * risk`
    pub fn select_action(
        &self,
        current: &Grid,
        target: Option<&Grid>,
        actions: &[Action],
        wm: &WorldModel,
        alpha: f32,
        beta: f32,
        gamma: f32,
        delta: f32,
    ) -> Option<Action> {
        let mut best: Option<(Action, f32)> = None;

        for action in actions {
            let av = self.compute_action_value(current, target, action, wm);
            let total = alpha * av.expected_information_gain
                + beta * av.expected_progress
                - gamma * av.action_cost
                - delta * av.risk;

            if best.is_none() || total > best.as_ref().unwrap().1 {
                best = Some((action.clone(), total));
            }
        }

        best.map(|(a, _)| a)
    }
}

/// Két grid pixel-egyezésének aránya 0.0 és 1.0 között.
fn pixel_match(a: &Grid, b: &Grid) -> f32 {
    if a.width != b.width || a.height != b.height {
        return 0.0;
    }
    let total = (a.width as usize) * (a.height as usize);
    if total == 0 {
        return 1.0;
    }
    let mut matching = 0usize;
    for i in 0..a.height as usize {
        for j in 0..a.width as usize {
            let idx = i * a.width as usize + j;
            let bidx = i * b.width as usize + j;
            if a.cells[idx] == b.cells[bidx] {
                matching += 1;
            }
        }
    }
    matching as f32 / total as f32
}
'''

s = s.rstrip() + "\n" + new_impl
write_file(p, s)
print("[1] Planner lib.rs bővítve: ActionValue, compute_action_value, select_action.")

# 2. Új teszt a planner crate-hez
test_code = r'''
use athlesia_planner::{Planner, PlannerMode, ActionValue};
use athlesia_world_model::WorldModel;
use athlesia_types::{Grid, Color, Action, PrimName, Params};

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
fn action_value_information_gain_is_positive() {
    let current = grid_from_rows(vec![vec![1, 0], vec![0, 0]]);
    let wm = WorldModel::new(current.clone());
    let planner = Planner::new(PlannerMode::Exploration);

    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let av = planner.compute_action_value(&current, None, &action, &wm);

    assert!(av.expected_information_gain >= 0.0 && av.expected_information_gain <= 1.0);
    assert_eq!(av.expected_progress, 0.0);
    assert_eq!(av.action_cost, 1.0);
    assert_eq!(av.risk, 0.0);
}

#[test]
fn select_action_uses_beta_for_progress() {
    let current = grid_from_rows(vec![vec![1, 0], vec![0, 0]]);
    let target = grid_from_rows(vec![vec![0, 1], vec![0, 0]]);
    let wm = WorldModel::new(current.clone());
    let planner = Planner::new(PlannerMode::GoalDirected);

    let actions = vec![
        Action { prim: PrimName::Translate, params: Params::Translate(1, 0) }, // javítja a progresszt
        Action { prim: PrimName::Translate, params: Params::Translate(0, 1) }, // rontja
    ];

    // Csak a progresszt nézzük (β=1, α=γ=δ=0)
    let best = planner.select_action(&current, Some(&target), &actions, &wm, 0.0, 1.0, 0.0, 0.0)
        .expect("Kell lennie kiválasztott akciónak");
    assert_eq!(best.params, Params::Translate(1, 0), "A jobb progresszű akciót kell választani");
}
'''

write_file("crates/athlesia-planner/tests/action_value_test.rs", test_code)
print("[2] action_value_test.rs létrehozva.")

# 3. Planner tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-planner"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Planner tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Phase 11 tesztek zöldek.")

# 4. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 11: ActionValue structure and weighted action selection in Planner"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
