use athlesia_kernel::solve_arc_json;
use athlesia_types::Grid;
use std::fs;

#[test]
fn measure_on_embedded_arc_tasks() {
    // Különböző, egyszerű ARC feladatok JSON-ben
    let tasks = vec![
        // 1. Jobbra tolás
        r#"{"train":[{"input":[[1,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]],"output":[[0,1,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]}],"test":[{"input":[[1,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]],"output":[[0,1,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]}]}"#,
        // 2. Vízszintes tükrözés
        r#"{"train":[{"input":[[1,2,3,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]],"output":[[3,2,1,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]}],"test":[{"input":[[1,2,3,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]],"output":[[3,2,1,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]}]}"#,
        // 3. Színcsere
        r#"{"train":[{"input":[[1,0,0,0,0],[0,2,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]],"output":[[2,0,0,0,0],[0,1,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]}],"test":[{"input":[[1,0,0,0,0],[0,2,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]],"output":[[2,0,0,0,0],[0,1,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]}]}"#,
    ];

    let mut solved = 0;
    let total = tasks.len();

    for (i, task_json) in tasks.iter().enumerate() {
        let (predicted, expected) = solve_arc_json(task_json);
        match predicted {
            Some(pred) if pred == expected => {
                println!("Feladat {}: OK", i + 1);
                solved += 1;
            }
            _ => println!("Feladat {}: FAILED", i + 1),
        }
    }

    println!("Összesen: {}/{} feladat megoldva.", solved, total);
    println!("Sikerességi arány: {:.0}%", (solved as f32 / total as f32) * 100.0);
}
