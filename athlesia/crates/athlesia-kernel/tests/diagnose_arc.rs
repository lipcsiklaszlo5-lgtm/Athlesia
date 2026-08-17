use std::fs;
use athlesia_search::{DefaultSearchEngine, SearchEngine, SearchStrategy};
use athlesia_kernel::grid_from_rows;

#[test]
fn diagnose_first_arc_task() {
    let task_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/ARC/data/training/007bbfb7.json");
    let content = fs::read_to_string(task_path).expect("Nem találom a feladatot");
    let task: serde_json::Value = serde_json::from_str(&content).unwrap();

    let train = task["train"].as_array().unwrap();
    let engine = DefaultSearchEngine;

    for (i, example) in train.iter().enumerate() {
        let input_rows: Vec<Vec<u8>> = example["input"]
            .as_array().unwrap().iter()
            .map(|row| row.as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u8).collect())
            .collect();
        let output_rows: Vec<Vec<u8>> = example["output"]
            .as_array().unwrap().iter()
            .map(|row| row.as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u8).collect())
            .collect();

        let input = grid_from_rows(&input_rows);
        let output = grid_from_rows(&output_rows);

        println!("Train példa {}: input {}x{}, output {}x{}", i+1, input.width, input.height, output.width, output.height);

        let solution = engine.search(&input, &output, 3, SearchStrategy::AStar);
        match solution {
            Some(program) => {
                println!("  Megoldás találva, hossz: {}", program.len());
                for (j, (prim, params)) in program.iter().enumerate() {
                    println!("    {}. {:?} {:?}", j+1, prim, params);
                }
            }
            None => {
                println!("  Nincs megoldás max_depth=3-ig.");
            }
        }
    }
}
