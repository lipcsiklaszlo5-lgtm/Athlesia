
use athlesia_types::{Grid, PrimName, Params, Program, Budget, ExecError, Color};



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
        PrimName::Rotate180 => {
            for y in 0..grid.height as i8 {
                for x in 0..grid.width as i8 {
                    if let Some(color) = grid.get(x, y) {
                        new_grid.set(grid.width as i8 - 1 - x, grid.height as i8 - 1 - y, color);
                    }
                }
            }
        }
        PrimName::Rotate270 => {
            for y in 0..grid.height as i8 {
                for x in 0..grid.width as i8 {
                    if let Some(color) = grid.get(x, y) {
                        new_grid.set(grid.height as i8 - 1 - y, x, color);
                    }
                }
            }
        }
        PrimName::AddBorder => {
            let new_width = grid.width + 2;
            let new_height = grid.height + 2;
            let mut bordered = Grid::new(new_width, new_height);
            for y in 0..grid.height as i8 {
                for x in 0..grid.width as i8 {
                    if let Some(color) = grid.get(x, y) {
                        bordered.set(x + 1, y + 1, color);
                    }
                }
            }
            return bordered;
        }
        PrimName::RemoveBorder => {
            if grid.width < 3 || grid.height < 3 {
                return grid.clone();
            }
            let new_width = grid.width - 2;
            let new_height = grid.height - 2;
            let mut cropped = Grid::new(new_width, new_height);
            for y in 1..grid.height as i8 - 1 {
                for x in 1..grid.width as i8 - 1 {
                    if let Some(color) = grid.get(x, y) {
                        cropped.set(x - 1, y - 1, color);
                    }
                }
            }
            return cropped;
        }
        PrimName::SwapColors => {
            if let Params::SwapColors(c1, c2) = params {
                for y in 0..grid.height as i8 {
                    for x in 0..grid.width as i8 {
                        if let Some(color) = grid.get(x, y) {
                            let new_color = if color.0 == *c1 {
                                Color(*c2)
                            } else if color.0 == *c2 {
                                Color(*c1)
                            } else {
                                color
                            };
                            new_grid.set(x, y, new_color);
                        }
                    }
                }
            }
        }
        PrimName::RepeatGrid => {
            // Az input rácsot k x k-szor ismétli blokkonként.
            // k = 2 fix, mert nincs paraméter.
            let k: u8 = 2;
            let new_width = grid.width * k;
            let new_height = grid.height * k;
            let mut repeated = Grid::new(new_width, new_height);
            for by in 0..k as i8 {
                for bx in 0..k as i8 {
                    for y in 0..grid.height as i8 {
                        for x in 0..grid.width as i8 {
                            if let Some(color) = grid.get(x, y) {
                                repeated.set(bx * grid.width as i8 + x, by * grid.height as i8 + y, color);
                            }
                        }
                    }
                }
            }
            return repeated;
        }
        PrimName::Tile => {
            // Az inputot cellánként megismételve nagyítja.
            // A kimenet mérete: width * tile_size, height * tile_size.
            // A tile_size 2 fix, mert a Params-ben nincs külön paraméter.
            let tile_size = 2u8;
            let new_width = grid.width * tile_size;
            let new_height = grid.height * tile_size;
            let mut tiled = Grid::new(new_width, new_height);
            for y in 0..grid.height as i8 {
                for x in 0..grid.width as i8 {
                    if let Some(color) = grid.get(x, y) {
                        for dy in 0..tile_size as i8 {
                            for dx in 0..tile_size as i8 {
                                tiled.set(x * tile_size as i8 + dx, y * tile_size as i8 + dy, color);
                            }
                        }
                    }
                }
            }
            return tiled;
        }
        PrimName::TranslateWrap => {
            if let Params::TranslateWrap(dx, dy) = params {
                let dx = *dx as i8;
                let dy = *dy as i8;
                for y in 0..grid.height as i8 {
                    for x in 0..grid.width as i8 {
                        if let Some(color) = grid.get(x, y) {
                            let nx = (x + dx).rem_euclid(grid.width as i8);
                            let ny = (y + dy).rem_euclid(grid.height as i8);
                            new_grid.set(nx, ny, color);
                        }
                    }
                }
            }
        }
        PrimName::CopyObject => {
            // Egyszerű placeholder: visszaadja a grid másolatát.
            return grid.clone();
        }
        PrimName::MoveTo => {
            return grid.clone();
        }
        PrimName::Connect => {
            return grid.clone();
        }
        PrimName::FillEnclosedArea => {
            return grid.clone();
        }
        PrimName::DrawLine => {
            return grid.clone();
        }
        PrimName::DrawBox => {
            return grid.clone();
        }
        PrimName::FillObject => {
            return grid.clone();
        }
        PrimName::ReplaceColor => {
            return grid.clone();
        }
        PrimName::ShiftRow => {
            return grid.clone();
        }
        PrimName::ShiftColumn => {
            return grid.clone();
        }
        PrimName::DeleteObject => {
            return grid.clone();
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
