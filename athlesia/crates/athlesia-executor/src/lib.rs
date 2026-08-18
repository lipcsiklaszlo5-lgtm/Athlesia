
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
        PrimName::BlockMap => {
            // BlockMap: a targetet input-méretű blokkokra bontja, és minden blokkra
            // a Paraméterben kapott transzformáció-azonosító listát alkalmazza.
            // A Params::BlockMap tartalmazza a sorok/oszlopok számát és a transzformációkat.
            if let Params::BlockMap(rows, cols, transforms) = params {
                let block_h = grid.height;
                let block_w = grid.width;
                let out_h = block_h * (*rows as u8);
                let out_w = block_w * (*cols as u8);
                let mut out = Grid::new(out_w, out_h);

                for r in 0..*rows {
                    for c in 0..*cols {
                        let idx = r * *cols + c;
                        let transform_id = transforms.get(idx).copied().unwrap_or(0u8);
                        let block = match transform_id {
                            1 => apply_primitive(grid, &PrimName::Rotate90, &Params::None),
                            2 => apply_primitive(grid, &PrimName::Rotate180, &Params::None),
                            3 => apply_primitive(grid, &PrimName::Rotate270, &Params::None),
                            4 => apply_primitive(grid, &PrimName::ReflectH, &Params::None),
                            5 => apply_primitive(grid, &PrimName::ReflectV, &Params::None),
                            _ => grid.clone(),
                        };

                        let start_x = (c as i8) * block_w as i8;
                        let start_y = (r as i8) * block_h as i8;
                        for y in 0..block_h as i8 {
                            for x in 0..block_w as i8 {
                                if let Some(cell) = block.get(x, y) {
                                    out.set(start_x + x, start_y + y, cell);
                                }
                            }
                        }
                    }
                }
                return out;
            }
            return grid.clone();
        }
        PrimName::RepeatGrid => {
            if let Params::RepeatGrid(k) = params {
                let k = *k as u8;
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
            return grid.clone();
        }
PrimName::Tile => {
            if let Params::Tile(tile_size) = params {
                let tile_size = *tile_size as u8;
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
            return grid.clone();
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
    if budget.max_depth == 0 {
        return Err(ExecError::DepthExceeded);
    }

    let mut current = input.clone();

    for (i, (name, params)) in program.iter().enumerate() {
        if budget.max_steps == 0 {
            return Err(ExecError::BudgetExceeded);
        }
        if i as u32 >= budget.max_depth {
            return Err(ExecError::DepthExceeded);
        }
        current = apply_primitive(&current, name, params);
        budget.max_steps -= 1;
    }

    Ok(current)
}
