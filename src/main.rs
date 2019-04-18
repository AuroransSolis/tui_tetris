extern crate termion;

use termion::{
    screen::AlternateScreen,
    input::TermRead,
    event::Key,
    color::{self, *}
};

mod tetromino;
mod gameboard;
mod game_config;

use tetromino::*;
use gameboard::*;
use game_config::*;

use std::fs::File;
use std::io::{stdout, Write, Stdout, stdin, Stdin};

fn main() {
    let mut screen = AlternateScreen::from(stdout());
    //let game_config = GameConfig::load_config(&mut screen);
    writeln!(screen, "Testing!\n").unwrap();
    write!(screen, "Testing!\nTesting!").unwrap();
    for key in stdin().keys() {
        if let Ok(_) = key {
            break;
        }
    }
}
