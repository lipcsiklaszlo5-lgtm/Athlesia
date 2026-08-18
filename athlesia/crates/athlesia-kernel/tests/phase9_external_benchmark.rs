
use athlesia_kernel::{ArcTask, grid_from_rows};
use athlesia_core::CoreEngine;
use std::path::Path;

fn load_task(path: &str) -> ArcTask {
    let content = std::fs::read_to_string(path).expect("Failed to read task file");
    serde_json::from_str(&content).expect("Invalid JSON")
}

fn solve_task(engine: &mut CoreEngine, task: &ArcTask) -> (bool, usize) {
    let mut total_steps = 0;
    let mut any_success = false;

    for example in &task.train {
        let input = grid_from_rows(&example.input);
        let output = grid_from_rows(&example.output);
        let (result, steps) = engine.solve_with_steps(&input, &output);
        total_steps += steps;
        if result.is_some() {
            any_success = true;
        }
    }

    for example in &task.test {
        let input = grid_from_rows(&example.input);
        let output = grid_from_rows(&example.output);
        let (result, steps) = engine.solve_with_steps(&input, &output);
        total_steps += steps;
        if result.is_some() {
            any_success = true;
        }
    }

    (any_success, total_steps)
}

#[test]
fn phase9_external_generalization_benchmark() {
    let base = concat!(env!("CARGO_MANIFEST_DIR"), "/../../benchmark/generalization/tasks/");
    let train_tasks = vec![
        "train_task_001.json",
        "train_task_002.json",
        "train_task_003.json",
    ];
    let heldout_task_name = "heldout_task_001.json";

    // Learning ON: egy engine, folyamatos tanulás a train taskokon
    let mut engine_on = CoreEngine::new();
    for task_name in &train_tasks {
        let path = format!("{}{}", base, task_name);
        let task = load_task(&path);
        let (success, steps) = solve_task(&mut engine_on, &task);
        assert!(success, "Training task {} should be solved", task_name);
        println!("Training task {} solved with {} steps", task_name, steps);
    }

    // Learning OFF: friss engine a heldout taskhoz (nem látott semmit)
    let mut engine_off = CoreEngine::new();
    let heldout_path = format!("{}{}", base, heldout_task_name);
    let heldout_task = load_task(&heldout_path);
    let (off_success, off_steps) = solve_task(&mut engine_off, &heldout_task);

    // Learning ON: ugyanaz a heldout task a már tanult modellel
    let (on_success, on_steps) = solve_task(&mut engine_on, &heldout_task);

    println!("Heldout task: learning OFF success={}, steps={}", off_success, off_steps);
    println!("Heldout task: learning ON success={}, steps={}", on_success, on_steps);

    // A kulcsfeltétel: ha tanulás nélkül nem sikerült, tanulással sikerülnie kell.
    // Ha mindkettő sikeres, akkor a lépésszámnak csökkennie kell.
    assert!(on_success, "After learning, the heldout task must be solved");
    if off_success {
        assert!(on_steps < off_steps, "Learning should reduce search cost: on_steps={}, off_steps={}", on_steps, off_steps);
    } else {
        println!("Heldout task was not solvable without learning; learning enabled it.");
    }
}
