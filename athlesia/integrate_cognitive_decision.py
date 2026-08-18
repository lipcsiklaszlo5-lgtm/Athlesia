#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. lib.rs beolvasása
p = pathlib.Path("crates/athlesia-kernel/src/lib.rs")
s = p.read_text()

# 2. Import hozzáadása a meglévő use-ok után
import_marker = "use athlesia_core::CoreEngine;"
if "use crate::cognitive::{CognitiveController, CognitiveDecision};" not in s:
    s = s.replace(import_marker, import_marker + "\nuse crate::cognitive::{CognitiveController, CognitiveDecision};")

# 3. solve_arc_json függvény cseréje
old_func_start = "pub fn solve_arc_json(task_json: &str) -> (Option<Grid>, Grid) {"
old_func_end = "/// Egyszerű kernel szintű megoldás adott bemenet-cél párra."
start_idx = s.find(old_func_start)
end_idx = s.find(old_func_end)
if start_idx == -1 or end_idx == -1:
    print("[ERROR] solve_arc_json függvény nem található a várt helyen.")
    sys.exit(1)

new_func = r'''pub fn solve_arc_json(task_json: &str) -> (Option<Grid>, Grid) {
    let task: ArcTask = serde_json::from_str(task_json).expect("Hibás ARC JSON");

    let first_train_input = grid_from_rows(&task.train[0].input);
    let first_train_output = grid_from_rows(&task.train[0].output);
    let features = athlesia_features::extract_features(&first_train_input);

    let mut agent = Agent::new(first_train_input.clone());

    // Kognitív döntés a feladat elején
    let decision = CognitiveController::decide(
        &features,
        &agent.core.meta,
        &agent.core.known_programs,
        &first_train_input,
        &first_train_output,
    );

    if decision == CognitiveDecision::Abstain {
        // Nem értjük, ne pazaroljunk keresést.
        let test_expected = grid_from_rows(&task.test[0].output);
        return (None, test_expected);
    }

    // Tanulás a train példákon (eredeti logika)
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

'''
s = s[:start_idx] + new_func + s[end_idx:]

write_file(p, s)
print("[1] solve_arc_json módosítva: kognitív döntés integrálva.")

# 4. Új teszt létrehozása
test_code = r'''
use athlesia_kernel::solve_arc_json;
use athlesia_kernel::grid_from_rows;

#[test]
fn test_solve_arc_json_abstain_when_no_prior() {
    let task_json = r#"{
        "train": [{"input": [[0,0],[0,0]], "output": [[1,1],[1,1]]}],
        "test": [{"input": [[0,0],[0,0]], "output": [[1,1],[1,1]]}]
    }"#;

    let (predicted, expected) = solve_arc_json(task_json);
    assert!(predicted.is_none());
    assert_eq!(expected, grid_from_rows(&vec![vec![1,1], vec![1,1]]));
}
'''
write_file("crates/athlesia-kernel/tests/cognitive_solve_test.rs", test_code)
print("[2] cognitive_solve_test.rs létrehozva.")

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
subprocess.run(["git", "commit", "-m", "Integrate cognitive decision into solve_arc_json; add abstain test"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
