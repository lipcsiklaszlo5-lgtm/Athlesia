
use athlesia_types::{Grid, PrimName, Params, Program, Budget, Color};
use athlesia_executor::run_program;

/// Keresési primitívek listája. A Synthesis Engine innen építkezik.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveTemplate {
    Translate,
    ReflectH,
    ReflectV,
    Rotate90,
    Recolor,
}

/// Elemi transzformációk generálása a template-hez.
/// Itt csak a lehetséges paraméterek egy rögzített, korlátozott halmazát adjuk vissza.
/// A cél a determinisztikus, korlátos keresés, nem az összes lehetséges paraméter.
fn generate_primitives(template: PrimitiveTemplate) -> Vec<(PrimName, Params)> {
    match template {
        PrimitiveTemplate::Translate => {
            // 4 környező irány + identitás
            let mut v = Vec::new();
            for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0), (0, 0)] {
                v.push((PrimName::Translate, Params::Translate(dx, dy)));
            }
            v
        }
        PrimitiveTemplate::ReflectH => {
            vec![(PrimName::ReflectH, Params::None)]
        }
        PrimitiveTemplate::ReflectV => {
            vec![(PrimName::ReflectV, Params::None)]
        }
        PrimitiveTemplate::Rotate90 => {
            vec![(PrimName::Rotate90, Params::None)]
        }
        PrimitiveTemplate::Recolor => {
            // Néhány gyakori permutáció
            let mut v = Vec::new();
            for perm in [
                [Color(1), Color(0), Color(2), Color(3)],
                [Color(2), Color(1), Color(0), Color(3)],
                [Color(3), Color(2), Color(1), Color(0)],
                [Color(1), Color(2), Color(3), Color(0)],
            ] {
                v.push((PrimName::Recolor, Params::Recolor(perm)));
            }
            v
        }
    }
}

/// Egyszerű, 1 lépéses program szintézis.
/// Megpróbál minden sablont és az azokhoz tartozó primitíveket,
/// és visszaadja az első olyan programot, ami a kívánt kimenetet adja.
pub fn synthesize(input: &Grid, target: &Grid, templates: &[PrimitiveTemplate]) -> Option<Program> {
    for template in templates {
        for (prim, params) in generate_primitives(*template) {
            let program = vec![(prim, params)];
            let mut budget = Budget { max_steps: 1 };
            if let Ok(output) = run_program(&program, input, &mut budget) {
                if output == *target {
                    return Some(program);
                }
            }
        }
    }
    None
}
