
use athlesia_kernel::{Agent, grid_from_rows};
use athlesia_types::{PrimName, Params};

#[test]
fn abstraction_from_episodes_adds_macro() {
    let mut agent = Agent::new(grid_from_rows(&vec![vec![0; 2]; 2]));

    let program = vec![(
        PrimName::BlockMap,
        Params::BlockMap(2, 2, vec![0, 0, 0, 0]),
    )];

    // Két példa ugyanarra a BlockMap programra
    agent.memory.append_episode(
        grid_from_rows(&vec![vec![1, 2], vec![3, 4]]),
        grid_from_rows(&vec![
            vec![1, 2, 1, 2],
            vec![3, 4, 3, 4],
            vec![1, 2, 1, 2],
            vec![3, 4, 3, 4],
        ]),
        program.clone(),
    );
    agent.memory.append_episode(
        grid_from_rows(&vec![vec![5, 6], vec![7, 8]]),
        grid_from_rows(&vec![
            vec![5, 6, 5, 6],
            vec![7, 8, 7, 8],
            vec![5, 6, 5, 6],
            vec![7, 8, 7, 8],
        ]),
        program.clone(),
    );

    agent.abstract_from_episodes();

    // Ellenőrizzük, hogy a makró bekerült a KB-be
    assert!(
        agent.kb.get_all_macros().iter().any(|m| m.program == program),
        "A makró nem található a tudásbázisban"
    );

    // Ellenőrizzük, hogy a CoreEngine ismeri a makrót
    assert!(agent.core.known_programs.contains(&program));
}
