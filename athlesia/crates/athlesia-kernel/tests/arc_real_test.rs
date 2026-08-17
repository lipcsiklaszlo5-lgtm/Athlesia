use athlesia_kernel::solve_arc_json;

#[test]
fn solve_embedded_arc_task() {
    // Egyszerű ARC feladat: egyetlen 1-es cella eltolása jobbra.
    // Train példa: input [[1,...]] -> output [[0,1,...]]
    let task_json = r#"
    {
      "train": [
        {
          "input": [[1,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]],
          "output": [[0,1,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]
        }
      ],
      "test": [
        {
          "input": [[1,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]],
          "output": [[0,1,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]
        }
      ]
    }
    "#;

    let (predicted, expected) = solve_arc_json(task_json);

    println!("Elvárt grid:");
    for row in expected.cells.iter() {
        println!("{:?}", row);
    }

    if let Some(pred) = predicted {
        println!("Prediktált grid:");
        for row in pred.cells.iter() {
            println!("{:?}", row);
        }
        assert_eq!(pred, expected, "A motornak meg kell oldania ezt az egyszerű ARC feladatot");
    } else {
        panic!("Nem sikerült prediktálni.");
    }
}
