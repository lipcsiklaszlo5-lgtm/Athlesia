pub const GRID_SIZE: usize = 5;
pub const N_COLORS: u8 = 4;
pub type Color = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grid { pub cells: [[Color; GRID_SIZE]; GRID_SIZE] }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimName { Translate, ReflectH, ReflectV, Rotate90, Recolor }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Params { None, Translate(i8, i8), Recolor([Color; 4]) }

pub type Program = Vec<(PrimName, Params)>;

#[derive(Debug, Clone, Copy)]
pub struct Budget { pub max_steps: u64 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecError { BudgetExceeded }


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coord {
    pub x: i8,
    pub y: i8,
}
