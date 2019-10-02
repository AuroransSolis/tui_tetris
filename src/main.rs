extern crate serde;
extern crate crossterm;

use termion::{
    color::{self, *},
    event::Key,
    input::TermRead,
    screen::AlternateScreen,
};

mod game_config;
mod gameboard;
mod tetromino;

use game_config::*;
use gameboard::*;
use tetromino::*;

use std::fs::File;
use std::io::{stdin, stdout, Read, Stdin, Stdout, Write};

fn main() {
    let game_config = if let Ok(mut file) = File::open("tui_tetris.conf") {
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents);
        match toml::from_str(&file_contents) {
            Ok(game_config_precursor) => game_config_precursor,
            Err(e) => {
                println!(
                    "Error loading game config from file! Location: {:?}\nDescription:\n{:?}",
                    e.line_col(),
                    e
                );
                return;
            }
        }
    } else {
        print!(
            "Could not find game config file! Would you like to use the default config? [y/N]: "
        );
    };
    let mut screen = AlternateScreen::from(stdout());
    writeln!(screen, "Testing!\n").unwrap();
    write!(screen, "Testing!\nTesting!").unwrap();
    for key in stdin().keys() {
        if let Ok(_) = key {
            break;
        }
    }
}
