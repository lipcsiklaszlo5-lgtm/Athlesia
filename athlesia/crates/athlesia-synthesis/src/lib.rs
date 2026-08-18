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
    Tile,
    RepeatGrid,
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
        PrimitiveTemplate::Tile => Vec::new(),
        PrimitiveTemplate::RepeatGrid => Vec::new(),
    }
}

pub fn synthesize(input: &Grid, target: &Grid, templates: &[PrimitiveTemplate]) -> Option<Program> {
    for template in templates {
        for (prim, params) in generate_primitives(*template) {
            let program = vec![(prim, params)];
            let mut budget = Budget { max_steps: 1, max_depth: 100 };
            if let Ok(output) = run_program(&program, input, &mut budget) {
                if output == *target {
                    return Some(program);
                }
            }
        }
    }

    // MetaGrid-alapú BlockMap generálás
    if let Some(meta) = athlesia_structure::TargetDecomposer::decompose_dimensions(input, target) {
        // A MetaGrid celláiból készítünk BlockMap paramétereket
        let mut transforms: Vec<u8> = Vec::new();
        // A decompose-ot használjuk a transzformációk felismerésére
        let decomposer = athlesia_structure::TargetDecomposer;
        if let Some(grid_meta) = decomposer.decompose(input, target) {
            for cell in &grid_meta.cells {
                transforms.push(match cell {
                    Some(athlesia_structure::TransformId::Identity) => 0,
                    Some(athlesia_structure::TransformId::Rot90) => 1,
                    Some(athlesia_structure::TransformId::Rot180) => 2,
                    Some(athlesia_structure::TransformId::Rot270) => 3,
                    Some(athlesia_structure::TransformId::ReflectH) => 4,
                    Some(athlesia_structure::TransformId::ReflectV) => 5,
                    None => return None,
                });
            }
            let program = vec![(
                PrimName::BlockMap,
                Params::BlockMap(meta.block_rows, meta.block_cols, transforms),
            )];
            let mut budget = Budget { max_steps: 1, max_depth: 100 };
            if let Ok(output) = run_program(&program, input, &mut budget) {
                if output == *target {
                    return Some(program);
                }
            }
        }
    }

    // ConditionalTile kipróbálása


    if target.width == input.width.saturating_mul(input.width) &&


       target.height == input.height.saturating_mul(input.height)


    {


        let program = vec![(PrimName::ConditionalTile, Params::ConditionalTile)];


        let mut budget = Budget { max_steps: 1, max_depth: 100 };


        if let Ok(output) = run_program(&program, input, &mut budget) {


            if output == *target {


                return Some(program);


            }


        }


    }


    // Dimenzióváltó primitívek induktív kipróbálása.
    // Például 3x3 -> 9x9 esetén RepeatGrid(3) vagy Tile(3).
    if input.width > 0 && input.height > 0 {
        let w_ratio = target.width / input.width;
        let h_ratio = target.height / input.height;
        if target.width % input.width == 0
            && target.height % input.height == 0
            && w_ratio == h_ratio
            && w_ratio > 0
        {
            let k = w_ratio as usize;
            let dim_programs = [
                vec![(PrimName::RepeatGrid, Params::RepeatGrid(k))],
                vec![(PrimName::Tile, Params::Tile(k))],
            ];
            for program in dim_programs.iter() {
                let mut budget = Budget { max_steps: 1, max_depth: 100 };
                if let Ok(output) = run_program(program, input, &mut budget) {
                    if output == *target {
                        return Some(program.clone());
                    }
                }
            }
        }
    }

    None
}
