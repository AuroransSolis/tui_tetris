use super::*;
use crossterm::Color;

struct Cell {
    character: char,
    colour: Color,
}

impl Cell {
    fn new(character: char, colour: Color) -> Self {
        Cell { character, colour }
    }
}

pub struct GameBoard {
    width: usize,
    height: usize,
    mode: Mode,
    cells: Vec<Cell>,
}

impl GameBoard {}
