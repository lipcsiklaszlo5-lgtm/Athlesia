
use athlesia_memory::Memory;
use athlesia_types::{Grid, PrimName, Params};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn stores_and_retrieves_program_by_exact_input() {
    let mut mem = Memory::new();
    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let target = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let program = vec![(PrimName::Translate, Params::Translate(1, 0))];

    mem.add_episode(input.clone(), target, program.clone());

    let retrieved = mem.find_program_by_input(&input);
    assert_eq!(retrieved, Some(program));

    assert_eq!(mem.get_known_programs().len(), 1);
}

#[test]
fn does_not_duplicate_known_program() {
    let mut mem = Memory::new();
    let input = build_grid([[0; 5]; 5]);
    let target = build_grid([[0; 5]; 5]);
    let program = vec![(PrimName::ReflectH, Params::None)];

    mem.add_episode(input.clone(), target.clone(), program.clone());
    mem.add_episode(input.clone(), target, program.clone());

    assert_eq!(mem.get_known_programs().len(), 1);
}
