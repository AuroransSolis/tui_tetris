use super::*;

struct Cell {
    character: char,
    colour: &'static dyn Color
}

impl Cell {
    fn new(character: char, colour: &'static dyn Color) -> Self {
        Cell {
            character,
            colour
        }
    }
}

pub struct GameBoard {
    width: usize,
    height: usize,
    mode: Mode,
    cells: Vec<Cell>
}

impl GameBoard {

}