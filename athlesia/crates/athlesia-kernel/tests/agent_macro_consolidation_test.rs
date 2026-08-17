
use athlesia_kernel::Agent;
use athlesia_knowledge::KnowledgeBase;
use athlesia_memory::Memory;
use athlesia_types::{Grid, PrimName, Params, Budget};
use athlesia_executor::run_program;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn consolidates_learned_macro_for_next_level() {
    let start = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let mut agent = Agent::new(start.clone());
    let mut kb = KnowledgeBase::new();
    let mut memory = Memory::new();

    // A környezet szabálya: jobbra tolás (Translate(1,0))
    let rule = vec![(PrimName::Translate, Params::Translate(1, 0))];

    // Első kör: tanulás és megerősítés
    let mut current = start;
    for _ in 0..5 {
        let _action = agent.step(&current, None);
        let mut budget = Budget { max_steps: 1 };
        let next = run_program(&rule, &current, &mut budget).unwrap();
        agent.update(&current, &next);
        current = next;
    }

    // Ellenőrizzük, hogy a hipotézis megerősített
    assert!(agent.wm.hypotheses.iter().any(|h| h.status == athlesia_world_model::HypothesisStatus::Confirmed));

    // Makrók konszolidálása a tudásbázisba
    agent.consolidate_learned_macros(&mut kb, &mut memory);

    assert!(kb.get_all_macros().len() > 0, "Legalább egy makrónak be kell kerülnie a tudásbázisba");
    assert!(memory.get_known_programs().len() > 0, "A memóriába is be kell kerülnie a megtanult szabálynak");
}
