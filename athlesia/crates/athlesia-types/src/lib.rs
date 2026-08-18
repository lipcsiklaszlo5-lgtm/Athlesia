
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Color(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Coord {
    pub x: i8,
    pub y: i8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grid {
    pub width: u8,
    pub height: u8,
    pub cells: Vec<Color>,
}

impl Grid {
    pub fn new(width: u8, height: u8) -> Self {
        Grid {
            width,
            height,
            cells: vec![Color(0); (width as usize) * (height as usize)],
        }
    }

    /// 5×5-ös teszt-adatokból hoz létre Grid-et.
    pub fn from_5x5(rows: [[u8; 5]; 5]) -> Self {
        let mut cells = Vec::with_capacity(25);
        for row in &rows {
            for &v in row {
                cells.push(Color(v));
            }
        }
        Grid { width: 5, height: 5, cells }
    }

    pub fn set(&mut self, x: i8, y: i8, color: Color) {
        if x >= 0 && x < self.width as i8 && y >= 0 && y < self.height as i8 {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            self.cells[idx] = color;
        }
    }

    pub fn get(&self, x: i8, y: i8) -> Option<Color> {
        if x >= 0 && x < self.width as i8 && y >= 0 && y < self.height as i8 {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            Some(self.cells[idx])
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimName {
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
    BlockMap,
    ConditionalTile,
    CopyObject,
    MoveTo,
    Connect,
    FillEnclosedArea,
    DrawLine,
    DrawBox,
    FillObject,
    ReplaceColor,
    ShiftRow,
    ShiftColumn,
    DeleteObject,}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Params {
    None,
    Translate(i8, i8),
    Recolor([Color; 10]),
    SwapColors(u8, u8),
    TranslateWrap(i8, i8),
    Tile(usize),
    RepeatGrid(usize),
    BlockMap(usize, usize, Vec<u8>),
    ConditionalTile,
}

pub type Program = Vec<(PrimName, Params)>;

#[derive(Debug, Clone, Copy)]
pub struct Budget {
    pub max_steps: u64,
    pub max_depth: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecError {
    BudgetExceeded,
    DepthExceeded,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Action {
    pub prim: PrimName,
    pub params: Params,
}


/// Kemény kényszer megsértése (például érvénytelen koordináta vagy dimenzió).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HardConstraintError {
    InvalidCoordinate { x: i8, y: i8 },
    InvalidDimensions { expected_width: u8, expected_height: u8 },
    InvalidColorValue { color: u8 },
    InvalidAction { action: String },
}
