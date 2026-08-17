
use athlesia_types::{Grid, PrimName, Params, Program, Budget, ExecError};



pub fn apply_primitive(grid: &Grid, name: &PrimName, params: &Params) -> Grid {
    let mut new_grid = Grid::new(grid.width, grid.height);

    match name {
        PrimName::Translate => {
            if let Params::Translate(dx, dy) = params {
                let dx = *dx as i8;
                let dy = *dy as i8;
                for y in 0..grid.height as i8 {
                    for x in 0..grid.width as i8 {
                        let (nx, ny) = (x + dx, y + dy);
                        if nx >= 0 && nx < grid.width as i8 && ny >= 0 && ny < grid.height as i8 {
                            if let Some(color) = grid.get(x, y) {
                                new_grid.set(nx, ny, color);
                            }
                        }
                    }
                }
            }
        }
        PrimName::ReflectH => {
            for y in 0..grid.height as i8 {
                for x in 0..grid.width as i8 {
                    if let Some(color) = grid.get(x, y) {
                        new_grid.set(grid.width as i8 - 1 - x, y, color);
                    }
                }
            }
        }
        PrimName::ReflectV => {
            for y in 0..grid.height as i8 {
                for x in 0..grid.width as i8 {
                    if let Some(color) = grid.get(x, y) {
                        new_grid.set(x, grid.height as i8 - 1 - y, color);
                    }
                }
            }
        }
        PrimName::Rotate90 => {
            for y in 0..grid.height as i8 {
                for x in 0..grid.width as i8 {
                    if let Some(color) = grid.get(x, y) {
                        new_grid.set(y, grid.width as i8 - 1 - x, color);
                    }
                }
            }
        }
        PrimName::Recolor => {
            if let Params::Recolor(perm) = params {
                for y in 0..grid.height as i8 {
                    for x in 0..grid.width as i8 {
                        if let Some(color) = grid.get(x, y) {
                            let new_color = perm[color.0 as usize];
                            new_grid.set(x, y, new_color);
                        }
                    }
                }
            }
        }
    }

    new_grid
}

pub fn run_program(program: &Program, input: &Grid, budget: &mut Budget) -> Result<Grid, ExecError> {
    let mut current = input.clone();

    for (name, params) in program {
        if budget.max_steps == 0 {
            return Err(ExecError::BudgetExceeded);
        }
        current = apply_primitive(&current, name, params);
        budget.max_steps -= 1;
    }

    Ok(current)
}
