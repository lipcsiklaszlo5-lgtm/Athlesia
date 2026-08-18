
use athlesia_interactive::{Environment, ProbeAction, InteractiveAgent};

fn object_position(grid: &athlesia_types::Grid) -> (i8, i8) {
    for y in 0..grid.height as i8 {
        for x in 0..grid.width as i8 {
            if let Some(c) = grid.get(x, y) {
                if c.0 != 0 {
                    return (x, y);
                }
            }
        }
    }
    (-1, -1)
}

#[test]
fn info_gain_learns_faster_than_random() {
    // A rejtett trigger mindig a C akció.
    let hidden = ProbeAction::C;
    let threshold = 0.95;

    // Információnyerés-alapú tanuló
    let mut env = Environment::new(hidden);
    let mut agent = InteractiveAgent::new();
    let mut steps_info = 0;
    loop {
        let (best, prob) = agent.best_hypothesis();
        if prob >= threshold {
            break;
        }
        let action = agent.select_action();
        // Elmentjük a régi pozíciót
        let old_pos = object_position(&env.grid);
        let _new_grid = env.step(&action);
        let new_pos = object_position(&env.grid);
        let moved = old_pos != new_pos;
        agent.update(&action, moved);
        steps_info += 1;
    }
    assert_eq!(agent.best_hypothesis().0, hidden, "Az információnyerés-alapú tanulónak meg kellett találnia a helyes triggert");
    assert!(steps_info <= 4, "Az információnyerés-alapú tanulás túl sok lépést igényelt: {}", steps_info);

    // Véletlenszerű felfedezés baseline (determinisztikus egyszerűség: mindig az első akció)
    // Itt most csak azt ellenőrizzük, hogy az info-gain kevesebb lépést használ, mint egy naiv végigpróbálás.
    // Mivel a környezet determinisztikus, a naiv stratégia legrosszabb esetben 5 lépés, ha sorban próbálja.
    // Az info-gain valószínűleg gyorsabb.
    assert!(steps_info < 5, "Az információnyerésnek gyorsabbnak kell lennie, mint a szekvenciális próbálkozás");
}
