use athlesia_types::{Grid, PrimName, Params, Program, Budget, Color};
use athlesia_executor::run_program;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveTemplate {
    Translate,
    ReflectH,
    ReflectV,
    Rotate90,
    Rotate180,
    Rotate270,
    Recolor,
    AddBorder,
    RemoveBorder,
    SwapColors,
    TranslateWrap,
}

fn generate_primitives(template: PrimitiveTemplate) -> Vec<(PrimName, Params)> {
    match template {
        PrimitiveTemplate::Translate => {
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
        PrimitiveTemplate::Rotate180 => {
            vec![(PrimName::Rotate180, Params::None)]
        }
        PrimitiveTemplate::Rotate270 => {
            vec![(PrimName::Rotate270, Params::None)]
        }
        PrimitiveTemplate::Recolor => {
            let mut v = Vec::new();
            let perms: [[Color; 10]; 4] = [
                [Color(1), Color(0), Color(2), Color(3), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)],
                [Color(2), Color(1), Color(0), Color(3), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)],
                [Color(3), Color(2), Color(1), Color(0), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)],
                [Color(1), Color(2), Color(3), Color(0), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)],
            ];
            for perm in perms {
                v.push((PrimName::Recolor, Params::Recolor(perm)));
            }
            v
        }
        PrimitiveTemplate::AddBorder => {
            vec![(PrimName::AddBorder, Params::None)]
        }
        PrimitiveTemplate::RemoveBorder => {
            vec![(PrimName::RemoveBorder, Params::None)]
        }
        PrimitiveTemplate::SwapColors => {
            vec![(PrimName::SwapColors, Params::SwapColors(1, 2)), (PrimName::SwapColors, Params::SwapColors(1, 3)), (PrimName::SwapColors, Params::SwapColors(2, 3))]
        }
        PrimitiveTemplate::TranslateWrap => {
            vec![(PrimName::TranslateWrap, Params::TranslateWrap(1, 0)), (PrimName::TranslateWrap, Params::TranslateWrap(0, 1)), (PrimName::TranslateWrap, Params::TranslateWrap(-1, 0)), (PrimName::TranslateWrap, Params::TranslateWrap(0, -1))]
        }
    }
}

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
