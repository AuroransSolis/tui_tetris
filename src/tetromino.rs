use crossterm::Color;
use rand::{Rng, rngs::ThreadRng};
use std::hint::unreachable_unchecked;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Tetromino {
    I,
    J,
    L,
    S,
    Z,
    T,
    O,
}

impl From<u16> for Tetromino {
    fn from(other: u16) -> Self {
        match other {
            0 => Tetromino::I,
            1 => Tetromino::J,
            2 => Tetromino::L,
            3 => Tetromino::S,
            4 => Tetromino::Z,
            5 => Tetromino::T,
            6 => Tetromino::O,
            _ => unsafe { unreachable_unchecked() }
        }
    }
}