
use athlesia_kernel::solve_arc_json;
use std::fs;

#[test]
fn real_arc_mini_benchmark() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/ARC/data/training");
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            files.push(path);
        }
    }
    files.sort();

    let total = 5.min(files.len());
    let mut solved = 0;

    for (i, path) in files.into_iter().take(total).enumerate() {
        let content = fs::read_to_string(&path).unwrap();
        let (predicted, expected) = solve_arc_json(&content);
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        if predicted == Some(expected) {
            solved += 1;
            println!("[OK] {} {}", i+1, name);
        } else {
            println!("[FAIL] {} {}", i+1, name);
        }
    }

    println!("Benchmark eredmény: {}/{} = {:.1}%", solved, total, (solved as f32 / total as f32) * 100.0);
}
