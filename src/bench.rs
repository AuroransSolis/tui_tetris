#[macro_use] extern crate criterion;
extern crate rand;
extern crate crossterm;

use rand::{thread_rng, Rng};

mod game_config;
mod gameboard;
mod tetromino;

use gameboard::decode_sequence_number;

use criterion::{Criterion, black_box};
use std::fs::read_to_string;
use game_config::GameConfig;

fn bench_decode_sequence_number(c: &mut Criterion) {
    let mut rng = thread_rng();
    c.bench_function("Decode tetromino sequence number", move |b| {
        b.iter_with_setup(|| rng.gen_range(0, 5040), |n| {
            black_box(decode_sequence_number(n));
        })
    });
}

fn bench_parse_game_config(c: &mut Criterion) {
    c.bench_function("Parse config file", move |b| {
        let file_string = read_to_string("tui_tetris.conf").unwrap();
        b.iter(|| {
            if let Err(e) = GameConfig::parse(file_string.as_str()) {
                panic!(e);
            }
        })
    });
}

criterion_group! {
    name = bench;
    config = Criterion::default();
    targets = bench_decode_sequence_number, bench_parse_game_config
}

criterion_main!{bench}