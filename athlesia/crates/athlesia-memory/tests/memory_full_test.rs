
use athlesia_memory::Memory;
use athlesia_types::{Grid, PrimName, Params};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn append_stores_episode_and_known_program() {
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

    mem.append(input, target, program.clone());

    assert_eq!(mem.episode_history().len(), 1);
    assert_eq!(mem.get_known_programs().len(), 1);
    assert_eq!(mem.snapshot(), vec![program]);
}

#[test]
fn working_context_is_set_and_cleared() {
    let mut mem = Memory::new();
    let grid = build_grid([[0; 5]; 5]);

    mem.set_working_context(grid.clone(), Some(42));
    assert!(mem.working.is_some());
    assert_eq!(mem.working.as_ref().unwrap().active_hypothesis, Some(42));

    mem.clear_working_context();
    assert!(mem.working.is_none());
}

#[test]
fn consolidate_known_programs_moves_all() {
    let mut mem = Memory::new();
    let input = build_grid([[0; 5]; 5]);
    let target = build_grid([[0; 5]; 5]);
    let program = vec![(PrimName::ReflectH, Params::None)];

    mem.append(input.clone(), target.clone(), program.clone());
    mem.append(input, target, program.clone());

    assert_eq!(mem.snapshot().len(), 1, "Duplikáció miatt eggyel kell lennie");

    mem.consolidate_known_programs();
    assert_eq!(mem.snapshot().len(), 1);
}
