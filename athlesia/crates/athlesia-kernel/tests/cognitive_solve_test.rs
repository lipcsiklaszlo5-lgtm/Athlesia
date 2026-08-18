
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
