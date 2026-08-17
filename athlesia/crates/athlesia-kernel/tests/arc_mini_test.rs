use std::fs;

#[test]
fn arc_data_files_are_present() {
    let task_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/arc-agi_training_challenges.json");
    let solution_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/arc-agi_training_solutions.json");

    let task_str = fs::read_to_string(task_path).expect("Nem találom a feladat JSON-t");
    let solution_str = fs::read_to_string(solution_path).expect("Nem találom a megoldás JSON-t");

    assert!(!task_str.is_empty());
    assert!(!solution_str.is_empty());

    println!("ARC feladat méret: {} bájt", task_str.len());
    println!("ARC megoldás méret: {} bájt", solution_str.len());
}
