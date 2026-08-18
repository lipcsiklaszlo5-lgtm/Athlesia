
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
