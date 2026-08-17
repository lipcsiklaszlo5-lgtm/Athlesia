use athlesia_memory::{Memory, InteractionEvent};
use athlesia_types::{Grid, PrimName, Params, Action};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn append_episode_stores_episode_and_known_program() {
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

    mem.append_episode(input, target, program.clone());

    assert_eq!(mem.episode_history().len(), 1);
    assert_eq!(mem.get_known_programs().len(), 1);
    assert_eq!(mem.snapshot(), vec![program]);
}

#[test]
fn interaction_log_records_events() {
    let mut mem = Memory::new();
    let grid = build_grid([[0; 5]; 5]);
    let action = Action { prim: PrimName::ReflectH, params: Params::None };

    mem.append_event(InteractionEvent::Observation(grid.clone()));
    mem.append_event(InteractionEvent::Action(action));
    mem.append_event(InteractionEvent::HypothesisConfirmed(42));

    assert_eq!(mem.interaction_history().len(), 3);
    assert!(matches!(&mem.interaction_history()[0], InteractionEvent::Observation(_)));
    assert!(matches!(&mem.interaction_history()[1], InteractionEvent::Action(_)));
    assert!(matches!(&mem.interaction_history()[2], InteractionEvent::HypothesisConfirmed(42)));
}

#[test]
fn consolidate_compresses_duplicates() {
    let mut mem = Memory::new();
    let input = build_grid([[0; 5]; 5]);
    let target = build_grid([[0; 5]; 5]);
    let program = vec![(PrimName::ReflectH, Params::None)];

    mem.append_episode(input.clone(), target.clone(), program.clone());
    mem.append_episode(input, target, program.clone());

    // Két epizód, de ugyanaz a program.
    mem.consolidate_known_programs();

    assert_eq!(mem.snapshot().len(), 1, "A tömörítés után csak egy egyedi program marad");
}
