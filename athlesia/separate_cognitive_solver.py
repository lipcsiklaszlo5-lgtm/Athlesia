#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. lib.rs visszaállítása az eredeti solve_arc_json-ra, és új függvény hozzáadása
p = pathlib.Path("crates/athlesia-kernel/src/lib.rs")
s = p.read_text()

# Eltávolítjuk a korábbi módosítást: visszaállítjuk az eredeti solve_arc_json-t
original_func_start = "pub fn solve_arc_json(task_json: &str) -> (Option<Grid>, Grid) {"
original_func_end = "/// Egyszerű kernel szintű megoldás adott bemenet-cél párra."
start_idx = s.find(original_func_start)
end_idx = s.find(original_func_end)
if start_idx == -1 or end_idx == -1:
    print("[ERROR] solve_arc_json függvény nem található.")
    sys.exit(1)

# Eredeti függvény visszaállítása (a jelenlegi módosított szakasz eltávolítása)
# Megkeressük a következő "/// Egyszerű" előtti részt, és lecseréljük az eredeti függvényre.
# Az egyszerűség kedvéért újraírjuk a teljes lib.rs-t? Nem, hanem a meglévő tartalomból kivágjuk a régi függvényt, és beillesztjük az eredetit.
# Az eredeti függvény szövegét betesszük.
original_solve_arc_json = '''pub fn solve_arc_json(task_json: &str) -> (Option<Grid>, Grid) {
    let task: ArcTask = serde_json::from_str(task_json).expect("Hibás ARC JSON");

    let mut agent = Agent::new(grid_from_rows(&task.train[0].input));

    // Tanulás a train példákon
    for example in &task.train {
        let input_grid = grid_from_rows(&example.input);
        let output_grid = grid_from_rows(&example.output);

        // Percepció
        let _perception = perceive(Some(&input_grid), &output_grid);

        // Cél-irányított lépés
        let action = agent.step(&input_grid, Some(&output_grid));
        let program = vec![(action.prim, action.params)];

        // Verifikáció
        let verifier = Verifier::new();
        if verifier.verify(&program, &vec![(input_grid.clone(), output_grid.clone())]) == VerificationResult::Accept {
            let id = agent.core.known_programs.len() as u64;
            agent.core.known_programs.push(program.clone());
            agent.core.meta.record_success_in_context(
                athlesia_features::extract_features(&input_grid),
                id,
            );
            agent.memory.append_episode(input_grid.clone(), output_grid.clone(), program);
        }
    }

    // Test predikció
    let test_input = grid_from_rows(&task.test[0].input);
    let test_expected = grid_from_rows(&task.test[0].output);

    let mut predicted = None;
    for program in agent.core.known_programs.iter().rev() {
        let mut budget = Budget { max_steps: 10, max_depth: 100 };
        if let Ok(output) = athlesia_executor::run_program(program, &test_input, &mut budget) {
            predicted = Some(output);
            break;
        }
    }

    (predicted, test_expected)
}

/// Kognitív döntéssel kiegészített megoldó: ha a rendszer nem érti a feladatot,
/// Abstain esetén None-t ad vissza.
pub fn solve_arc_json_with_cognition(task_json: &str) -> (Option<Grid>, Grid) {
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

    // Különben ugyanaz, mint a solve_arc_json
    solve_arc_json(task_json)
}
'''

# A meglévő szakasz cseréje
s = s[:start_idx] + original_solve_arc_json + s[end_idx:]

write_file(p, s)
print("[1] lib.rs visszaállítva, új solve_arc_json_with_cognition hozzáadva.")

# 2. cognitive_solve_test.rs frissítése, hogy az új függvényt használja
test_code = r'''
use athlesia_kernel::solve_arc_json_with_cognition;
use athlesia_kernel::grid_from_rows;

#[test]
fn test_solve_arc_json_with_cognition_abstain_when_no_prior() {
    let task_json = r#"{
        "train": [{"input": [[0,0],[0,0]], "output": [[1,1],[1,1]]}],
        "test": [{"input": [[0,0],[0,0]], "output": [[1,1],[1,1]]}]
    }"#;

    let (predicted, expected) = solve_arc_json_with_cognition(task_json);
    assert!(predicted.is_none());
    assert_eq!(expected, grid_from_rows(&vec![vec![1,1], vec![1,1]]));
}
'''
write_file("crates/athlesia-kernel/tests/cognitive_solve_test.rs", test_code)
print("[2] cognitive_solve_test.rs frissítve.")

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
subprocess.run(["git", "commit", "-m", "Separate cognitive solver to preserve existing tests; add solve_arc_json_with_cognition"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
