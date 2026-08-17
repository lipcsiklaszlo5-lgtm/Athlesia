use athlesia_types::{Grid, PrimName, Params, Program, ExecError, GRID_SIZE};
pub use athlesia_types::Budget;

pub fn apply_primitive(grid: &Grid, name: &PrimName, params: &Params) -> Grid {
    let mut new_grid = Grid { cells: [[0; GRID_SIZE]; GRID_SIZE] };
    match name {
        PrimName::Translate => {
            if let Params::Translate(dx, dy) = params {
                let (dx, dy) = (*dx as i8, *dy as i8);
                for i in 0..GRID_SIZE as i8 {
                    for j in 0..GRID_SIZE as i8 {
                        let (ni, nj) = (i + dy, j + dx);
                        if ni >= 0 && ni < GRID_SIZE as i8 && nj >= 0 && nj < GRID_SIZE as i8 {
                            new_grid.cells[ni as usize][nj as usize] = grid.cells[i as usize][j as usize];
                        }
                    }
                }
            }
        }
        PrimName::ReflectH => {
            for i in 0..GRID_SIZE {
                for j in 0..GRID_SIZE {
                    new_grid.cells[i][j] = grid.cells[i][GRID_SIZE - 1 - j];
                }
            }
        }
        PrimName::ReflectV => {
            for i in 0..GRID_SIZE {
                for j in 0..GRID_SIZE {
                    new_grid.cells[i][j] = grid.cells[GRID_SIZE - 1 - i][j];
                }
            }
        }
        PrimName::Rotate90 => {
            for i in 0..GRID_SIZE {
                for j in 0..GRID_SIZE {
                    new_grid.cells[i][j] = grid.cells[j][GRID_SIZE - 1 - i];
                }
            }
        }
        PrimName::Recolor => {
            if let Params::Recolor(perm) = params {
                for i in 0..GRID_SIZE {
                    for j in 0..GRID_SIZE {
                        let old = grid.cells[i][j] as usize;
                        new_grid.cells[i][j] = perm[old];
                    }
                }
            }
        }
    }
    new_grid
}

pub fn run_program(program: &Program, input: &Grid, budget: &mut Budget) -> Result<Grid, ExecError> {
    let mut current = *input;
    for (name, params) in program {
        if budget.max_steps == 0 { return Err(ExecError::BudgetExceeded); }
        current = apply_primitive(&current, name, params);
        budget.max_steps -= 1;
    }
    Ok(current)
}
