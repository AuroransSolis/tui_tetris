use crossterm::Color;
use rand::{thread_rng, rngs::ThreadRng, Rng};

use crate::game_config::{GameConfig, Mode};
use crate::tetromino::Tetromino;
use std::hint::unreachable_unchecked;

struct Cell {
    character: char,
    colour: Color,
}

impl Cell {
    fn new(character: char, colour: Color) -> Self {
        Cell { character, colour }
    }
}

struct GameBoard {
    width: usize,
    height: usize,
    cells: Vec<Option<Cell>>,
    active_piece: [usize; 4]
}

impl GameBoard {
    fn new(width: usize, height: usize) -> Self {
        GameBoard {
            width,
            height,
            cells: Vec::with_capacity(width * height),
            active_piece: [0; 4]
        }
    }

    // Placeholder until I get around to learning how to use crossterm better
    fn draw(&self) {

    }
}

pub struct Game {
    config: GameConfig,
    board: GameBoard,
    rng: ThreadRng,
    sequence: [Tetromino; 7],
    sequence_ind: usize,
    score: u64,
    preview: Option<[Tetromino; 4]>,
    hold: Option<Tetromino>,
    level: usize,
    lines_cleared: usize
}

impl Game {
    pub fn new(config: GameConfig) -> Self {
        let mut rng = thread_rng();
        let board = GameBoard::new(config.board_width, config.board_height);
        let sequence = decode_sequence_number(rng.gen_range(0, 5040));
        let preview = match config.mode {
            Mode::Modern => Some({
                let mut preview = [Tetromino::I; 4];
                preview.copy_from_slice(&sequence[0..4]);
                preview
            }),
            Mode::Classic => None
        };
        Game {
            config,
            board,
            rng,
            sequence,
            sequence_ind: 0,
            score: 0,
            preview,
            hold: None,
            level: 0,
            lines_cleared: 0
        }
    }
}

// Generate the piece sequence by the following algorithm:
// input: sequence_number: usize (sn), usage_map: [bool; 7] (um, true = used, false = unused)
// for piece n:
//     piece number = nth false in um, where n is sequence number / (n - 1)!
//     sequence number -= piece number * (n - 1)!
// This iterative process has been unrolled and the range testing hard-coded.
pub fn decode_sequence_number(mut sequence_number: u16) -> [Tetromino; 7] {
    let mut in_use = [false; 7];
    let (p0, subtract) = match sequence_number {
        _ if (0..720).contains(&sequence_number) => (0, 0),
        _ if (720..1440).contains(&sequence_number) => (1, 720),
        _ if (1440..2160).contains(&sequence_number) => (2, 1440),
        _ if (2160..2880).contains(&sequence_number) => (3, 2160),
        _ if (2880..3600).contains(&sequence_number) => (4, 2880),
        _ if (3600..4320).contains(&sequence_number) => (5, 3600),
        _ if (4320..5040).contains(&sequence_number) => (6, 4320),
        _ => unsafe { unreachable_unchecked() }
    };
    sequence_number -= subtract;
    in_use[p0 as usize] = true;
    let (mut p1, subtract) = match sequence_number {
        _ if (0..120).contains(&sequence_number) => (0, 0),
        _ if (120..240).contains(&sequence_number) => (1, 120),
        _ if (240..360).contains(&sequence_number) => (2, 240),
        _ if (360..480).contains(&sequence_number) => (3, 360),
        _ if (480..600).contains(&sequence_number) => (4, 480),
        _ if (600..720).contains(&sequence_number) => (5, 600),
        _ => unsafe { unreachable_unchecked() }
    };
    sequence_number -= subtract;
    p1 = find_nth_unused(in_use, p1 as usize);
    in_use[p1 as usize] = true;
    let (mut p2, subtract) = match sequence_number {
        _ if (0..24).contains(&sequence_number) => (0, 0),
        _ if (24..48).contains(&sequence_number) => (1, 24),
        _ if (48..72).contains(&sequence_number) => (2, 48),
        _ if (72..96).contains(&sequence_number) => (3, 72),
        _ if (96..120).contains(&sequence_number) => (4, 96),
        _ => unsafe { unreachable_unchecked() }
    };
    sequence_number -= subtract;
    p2 = find_nth_unused(in_use, p2 as usize);
    in_use[p2 as usize] = true;
    let (mut p3, subtract) = match sequence_number {
        _ if (0..6).contains(&sequence_number) => (0, 0),
        _ if (6..12).contains(&sequence_number) => (1, 6),
        _ if (12..18).contains(&sequence_number) => (2, 12),
        _ if (18..24).contains(&sequence_number) => (3, 18),
        _ => unsafe { unreachable_unchecked() }
    };
    sequence_number -= subtract;
    p3 = find_nth_unused(in_use, p3 as usize);
    in_use[p3 as usize] = true;
    let (mut p4, subtract) = match sequence_number {
        _ if (0..2).contains(&sequence_number) => (0, 0),
        _ if (2..4).contains(&sequence_number) => (1, 2),
        _ if (4..6).contains(&sequence_number) => (2, 4),
        _ => unsafe { unreachable_unchecked() }
    };
    sequence_number -= subtract;
    p4 = find_nth_unused(in_use, p4 as usize);
    in_use[p4 as usize] = true;
    let p5 = find_nth_unused(in_use, sequence_number as usize);
    in_use[p5 as usize] = true;
    let mut p6 = 0;
    while in_use[p6 as usize] {
        p6 += 1;
    }
    [
        Tetromino::from(p0),
        Tetromino::from(p1),
        Tetromino::from(p2),
        Tetromino::from(p3),
        Tetromino::from(p4),
        Tetromino::from(p5),
        Tetromino::from(p6)
    ]
}

// Test to ensure that no input in the input space (0..5040) gives an output with (a) duplicate
// tetromino(s).
#[test]
fn test_sequence_decode() {
    for n in 0..5040 {
        let sequence = decode_sequence_number(n);
        for i in 0..6 {
            for j in i + 1..7 {
                if sequence[i] == sequence[j] {
                    let message = format!(
                        "Duplicate tetromino in sequence for sn {}: {:?}",
                        n, sequence
                    );
                    panic!(message);
                }
            }
        }
    }
}

// Ensure that all output values are unique for the input range (0..5040).
#[test]
fn test_no_duplicate_sequences() {
    let mut sequences = [[Tetromino::I; 7]; 5040];
    for n in 0..5040 {
        sequences[n] = decode_sequence_number(n as u16);
    }
    for i in 0..5039 {
        for j in i + 1..5040 {
            if sequences[i] == sequences[j] {
                let message = format!(
                    "Duplicate sequence for sns {} and {}: {:?}",
                    i, j, sequences[i]
                );
            }
        }
    }
}

fn find_nth_unused(usage_map: [bool; 7], mut n: usize) -> u16 {
    let mut ind = 0;
    while n > 0 || usage_map[ind] {
        if !usage_map[ind] {
            n -= 1;
        }
        ind += 1;
    }
    ind as u16
}