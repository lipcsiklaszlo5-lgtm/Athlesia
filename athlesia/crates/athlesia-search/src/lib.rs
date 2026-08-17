
use athlesia_types::{Grid, PrimName, Params, Program, Budget};
use athlesia_executor::run_program;

/// Determinisztikus, korlátos mélységű programkeresés.
/// A cél: olyan programot találni, amely az inputból a cél gridet állítja elő.
/// A keresés a lehetséges primitívek kombinációit próbálja ki,
/// de nem az összeset, hanem egy rögzített, korlátozott paraméterhalmazt.
///
/// A keresés mélysége `max_depth` lépés. Minden lépésben minden primitív kipróbálható.
/// Determinisztikus, mert a primitívek listája és a bejárás sorrendje rögzített.

fn candidate_primitives() -> Vec<(PrimName, Params)> {
    let mut v = Vec::new();

    // Eltolások: 4 irány + identitás
    for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0), (0, 0)] {
        v.push((PrimName::Translate, Params::Translate(dx, dy)));
    }

    // Tükrözések
    v.push((PrimName::ReflectH, Params::None));
    v.push((PrimName::ReflectV, Params::None));

    // Forgás
    v.push((PrimName::Rotate90, Params::None));

    // Néhány színpermutáció
    for perm in [
        [1, 0, 2, 3],
        [2, 1, 0, 3],
        [3, 2, 1, 0],
        [1, 2, 3, 0],
        [0, 1, 2, 3],
    ] {
        v.push((PrimName::Recolor, Params::Recolor(perm)));
    }

    v
}

/// Rekurzív keresés: a `depth` hátralévő lépés számát jelzi.
/// A `current` a jelenlegi program, a `input` az eredeti rács, a `target` a cél.
fn dfs(
    input: &Grid,
    target: &Grid,
    max_depth: usize,
    depth: usize,
    current: &mut Program,
    budget: &mut Budget,
) -> Option<Program> {
    if depth == max_depth {
        // Kiértékeljük a teljes programot
        let mut b = Budget { max_steps: max_depth as u64 };
        if let Ok(output) = run_program(current, input, &mut b) {
            if output == *target {
                return Some(current.clone());
            }
        }
        return None;
    }

    for (prim, params) in candidate_primitives() {
        current.push((prim, params));
        if let Some(found) = dfs(input, target, max_depth, depth + 1, current, budget) {
            return Some(found);
        }
        current.pop();
    }
    None
}

/// Nyilvános kereső: iterál a mélységeken, és visszaadja az első találatot.
pub fn search(input: &Grid, target: &Grid, max_depth: usize) -> Option<Program> {
    for d in 1..=max_depth {
        let mut program = Vec::new();
        let mut budget = Budget { max_steps: d as u64 };
        if let Some(p) = dfs(input, target, d, 0, &mut program, &mut budget) {
            return Some(p);
        }
    }
    None
}
