#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. lib.rs beolvasása
p = pathlib.Path("crates/athlesia-kernel/src/lib.rs")
s = p.read_text()

# 2. Import hozzáadása a synthesis crate-hez
import_marker = "use athlesia_core::CoreEngine;"
if "use athlesia_synthesis::{synthesize, PrimitiveTemplate};" not in s:
    s = s.replace(import_marker, import_marker + "\nuse athlesia_synthesis::{synthesize, PrimitiveTemplate};")

# 3. solve_arc_json_with_cognition függvény cseréje
start_marker = "pub fn solve_arc_json_with_cognition(task_json: &str) -> (Option<Grid>, Grid) {"
end_marker = "/// Egyszerű kernel szintű megoldás adott bemenet-cél párra."
start_idx = s.find(start_marker)
end_idx = s.find(end_marker)
if start_idx == -1 or end_idx == -1:
    print("[ERROR] Nem található a solve_arc_json_with_cognition függvény vagy a következő szakasz.")
    sys.exit(1)

new_func = r'''pub fn solve_arc_json_with_cognition(task_json: &str) -> (Option<Grid>, Grid) {
    let task: ArcTask = serde_json::from_str(task_json).expect("Hibás ARC JSON");

    let first_train_input = grid_from_rows(&task.train[0].input);
    let first_train_output = grid_from_rows(&task.train[0].output);
    let features = athlesia_features::extract_features(&first_train_input);

    let mut agent = Agent::new(first_train_input.clone());

    let decision = CognitiveController::decide(
        &features,
        &agent.core.meta,
        &agent.core.known_programs,
        &first_train_input,
        &first_train_output,
    );

    if decision == CognitiveDecision::Abstain {
        let test_expected = grid_from_rows(&task.test[0].output);
        return (None, test_expected);
    }

    // Ha a strukturális egyezés magas, próbáljunk közvetlenül BlockMap programot generálni
    let estimate = CognitiveController::estimate(
        &features,
        &agent.core.meta,
        &first_train_input,
        &first_train_output,
    );

    if estimate.structural_match > 0.9 {
        // Az üres template lista a BlockMap generálást engedi (a synthesis crate így működik)
        let templates: &[PrimitiveTemplate] = &[];
        if let Some(program) = synthesize(&first_train_input, &first_train_output, templates) {
            let test_input = grid_from_rows(&task.test[0].input);
            let mut budget = Budget { max_steps: 10, max_depth: 100 };
            if let Ok(output) = athlesia_executor::run_program(&program, &test_input, &mut budget) {
                let test_expected = grid_from_rows(&task.test[0].output);
                return (Some(output), test_expected);
            }
        }
    }

    // Különben ugyanaz, mint a solve_arc_json
    solve_arc_json(task_json)
}

'''
s = s[:start_idx] + new_func + s[end_idx:]

write_file(p, s)
print("[1] lib.rs frissítve: solve_arc_json_with_cognition közvetlen BlockMap generálással.")

# 4. Új teszt létrehozása
test_code = r'''
use athlesia_kernel::solve_arc_json_with_cognition;
use athlesia_kernel::grid_from_rows;

#[test]
fn structured_blockmap_solve() {
    let task_json = r#"{
        "train": [{"input": [[1,2],[3,4]], "output": [[1,2,1,2],[3,4,3,4],[1,2,1,2],[3,4,3,4]]}],
        "test": [{"input": [[5,6],[7,8]], "output": [[5,6,5,6],[7,8,7,8],[5,6,5,6],[7,8,7,8]]}]
    }"#;

    let (predicted, expected) = solve_arc_json_with_cognition(task_json);
    assert!(predicted.is_some(), "A rendszernek meg kell oldania a strukturált feladatot");
    assert_eq!(predicted.unwrap(), expected);
}
'''
write_file("crates/athlesia-kernel/tests/structured_solve_test.rs", test_code)
print("[2] structured_solve_test.rs létrehozva.")

# 5. Tesztek futtatása
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

# 6. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 3: direct BlockMap synthesis when structural match is high"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
