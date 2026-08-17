
use athlesia_kernel::Agent;
use athlesia_types::{Grid, PrimName, Params, Budget};
use athlesia_executor::run_program;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn agent_learns_translate_rule_interactively() {
    let start = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let mut agent = Agent::new(start.clone());

    // A környezet rejtett szabálya: jobbra tolás (Translate(1,0))
    let rule = vec![(PrimName::Translate, Params::Translate(1, 0))];

    let mut current = start;
    for _ in 0..5 {
        // Az ágens kiválaszt egy akciót a jelenlegi állapot alapján
        let _action = agent.step(&current, None);

        // A környezet a valós szabályt alkalmazza, nem az ágens akcióját
        let mut budget = Budget { max_steps: 1, max_depth: 100 };
        let next = run_program(&rule, &current, &mut budget).unwrap();

        // Az ágens frissíti a világmodelljét a megfigyelés alapján
        agent.update(&current, &next);

        current = next;
    }

    // A Translate(1,0) hipotézisnek meg kell erősödnie
    let translate_hyp = agent
        .wm
        .hypotheses
        .iter()
        .find(|h| h.program == rule)
        .expect("A Translate(1,0) hipotézisnek léteznie kell");
    assert!(translate_hyp.evidence_for > 0, "A helyes hipotézisnek legyen pozitív megerősítése");

    // Legalább egy hibás hipotézist cáfolni kell
    let has_falsified = agent.wm.hypotheses.iter().any(|h| h.evidence_against > 0);
    assert!(has_falsified, "Legalább egy hibás hipotézist cáfolni kell");
}
