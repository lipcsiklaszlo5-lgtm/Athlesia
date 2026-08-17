use athlesia_executor::{run_program, Budget};
use athlesia_types::{Grid, PrimName, Params, GRID_SIZE};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct GoldenVector {
    name: String,
    program: Vec<(String, serde_json::Value)>,
    input: Vec<Vec<u8>>,
    expected_output: Vec<Vec<u8>>,
}

fn grid_from_vec(v: &[Vec<u8>]) -> Grid {
    let mut cells = [[0u8; GRID_SIZE]; GRID_SIZE];
    for i in 0..GRID_SIZE { for j in 0..GRID_SIZE { cells[i][j] = v[i][j]; } }
    Grid { cells }
}

#[test]
fn golden_vectors() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/golden_vectors.json");
    let content = fs::read_to_string(path).expect("Failed to read golden_vectors.json");
    let cases: Vec<GoldenVector> = serde_json::from_str(&content).expect("Invalid JSON");

    let mut failed = 0;
    for case in &cases {
        let program: Vec<(PrimName, Params)> = case.program.iter().map(|(op, params)| {
            let prim = match op.as_str() {
                "translate" => PrimName::Translate,
                "reflect_h" => PrimName::ReflectH,
                "reflect_v" => PrimName::ReflectV,
                "rotate90" => PrimName::Rotate90,
                "recolor" => PrimName::Recolor,
                other => panic!("Unknown primitive: {}", other),
            };
            let parsed_params = match prim {
                PrimName::Translate => {
                    let arr = params.as_array().expect("Translate params should be array");
                    Params::Translate(arr[0].as_i64().unwrap() as i8, arr[1].as_i64().unwrap() as i8)
                }
                PrimName::Recolor => {
                    let arr = params.as_array().expect("Recolor params should be array");
                    let mut perm = [0u8; 4];
                    for (i, v) in arr.iter().enumerate() { perm[i] = v.as_u64().unwrap() as u8; }
                    Params::Recolor(perm)
                }
                _ => Params::None,
            };
            (prim, parsed_params)
        }).collect();

        let input_grid = grid_from_vec(&case.input);
        let expected_grid = grid_from_vec(&case.expected_output);
        let mut budget = Budget { max_steps: 1000 };

        let result = run_program(&program, &input_grid, &mut budget);
        let success = match &result {
            Ok(output) => *output == expected_grid,
            Err(_) => false,
        };

        if !success {
            failed += 1;
            println!("FAIL: {}", case.name);
            match &result {
                Ok(output) => {
                    println!("Expected:
{:?}", expected_grid);
                    println!("Got:
{:?}", output);
                }
                Err(e) => println!("Error: {:?}", e),
            }
        }
    }
    assert_eq!(failed, 0, "{} golden test(s) failed", failed);
}
