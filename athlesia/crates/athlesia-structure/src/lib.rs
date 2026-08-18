
use athlesia_types::{Grid, PrimName, Params};
use athlesia_executor::apply_primitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockDecomposition {
    pub block_rows: usize,
    pub block_cols: usize,
    pub block_width: usize,
    pub block_height: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransformId {
    Identity,
    Rot90,
    Rot180,
    Rot270,
    ReflectH,
    ReflectV,
}

#[derive(Debug, Clone)]
pub struct MetaGrid {
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<Option<TransformId>>,
}

pub struct TargetDecomposer;

impl TargetDecomposer {
    pub fn decompose_dimensions(input: &Grid, target: &Grid) -> Option<BlockDecomposition> {
        if input.width == 0 || input.height == 0 {
            return None;
        }
        let block_cols = target.width as usize / input.width as usize;
        let block_rows = target.height as usize / input.height as usize;
        if block_cols == 0 || block_rows == 0 {
            return None;
        }
        if target.width as usize % input.width as usize != 0
            || target.height as usize % input.height as usize != 0
        {
            return None;
        }
        Some(BlockDecomposition {
            block_rows,
            block_cols,
            block_width: input.width as usize,
            block_height: input.height as usize,
        })
    }

    fn extract_block(target: &Grid, decomp: &BlockDecomposition, r: usize, c: usize) -> Grid {
        let mut block = Grid::new(decomp.block_width as u8, decomp.block_height as u8);
        let start_x = c * decomp.block_width;
        let start_y = r * decomp.block_height;
        for y in 0..decomp.block_height {
            for x in 0..decomp.block_width {
                if let Some(cell) = target.get((start_x + x) as i8, (start_y + y) as i8) {
                    block.set(x as i8, y as i8, cell);
                }
            }
        }
        block
    }

    fn match_block(input: &Grid, block: &Grid) -> Option<TransformId> {
        if *block == *input {
            return Some(TransformId::Identity);
        }

        let transforms = [
            (TransformId::Rot90, PrimName::Rotate90),
            (TransformId::Rot180, PrimName::Rotate180),
            (TransformId::Rot270, PrimName::Rotate270),
            (TransformId::ReflectH, PrimName::ReflectH),
            (TransformId::ReflectV, PrimName::ReflectV),
        ];

        for (id, prim) in transforms {
            let transformed = apply_primitive(input, &prim, &Params::None);
            if transformed == *block {
                return Some(id);
            }
        }

        None
    }

    pub fn decompose(&self, input: &Grid, target: &Grid) -> Option<MetaGrid> {
        let decomp = Self::decompose_dimensions(input, target)?;
        let mut cells = Vec::with_capacity(decomp.block_rows * decomp.block_cols);

        for r in 0..decomp.block_rows {
            for c in 0..decomp.block_cols {
                let block = Self::extract_block(target, &decomp, r, c);
                let transform = Self::match_block(input, &block);
                cells.push(transform);
            }
        }

        Some(MetaGrid {
            rows: decomp.block_rows,
            cols: decomp.block_cols,
            cells,
        })
    }
}
