extern crate crossterm;
extern crate serde;

mod game_config;
mod gameboard;
mod tetromino;

use game_config::*;
use gameboard::*;
use tetromino::*;

use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::Path;

fn main() {
    let game_config = if Path::new("./tui_tetris.conf").exists() {
        match read_to_string("./tui_tetris.conf") {
            Ok(contents) => match GameConfig::parse(contents.as_str()) {
                Ok(game_config) => game_config,
                Err(e) => {
                    println!("{}", e);
                    return;
                }
            },
            Err(e) => {
                println!("Critical error! Failed to read config file.\n{:?}", e);
                return;
            }
        }
    } else {
        let game_config = GameConfig::default();
        println!("Warning: using default game config.");
        match File::create(Path::new("./tui_tetris.conf")) {
            Ok(mut file) => match game_config.write_to_file(&mut file) {
                Ok(()) => println!("Created new config file and wrote default config."),
                Err(e) => {
                    println!(
                        "Critical error! Failed to write default config to new config file!\n\
                         {:?}",
                        e
                    );
                    return;
                }
            },
            Err(e) => {
                println!("Critical error! Failed to create new config file.\n{:?}", e);
                return;
            }
        }
        game_config
    };
}
