use super::*;
use crossterm::Color;

enum Shape {
    I,
    J,
    L,
    S,
    Z,
    T,
    O,
}

struct Tetromino {
    c0: (usize, usize),
    c1: (usize, usize),
    c2: (usize, usize),
    c3: (usize, usize),
    color: Color,
}

impl Tetromino {
    pub fn new(shape: Shape, board: &mut GameBoard) {}
}
