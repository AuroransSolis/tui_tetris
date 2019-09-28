use super::*;
use std::io::Read;
use std::hint::unreachable_unchecked;
use std::str::FromStr;
use std::collections::HashMap;
use crossterm::{Color, InputEvent, KeyEvent};

const CONFIG_OPTIONS: [&str; 35] = [
    "fps",
    "board_width",
    "board_height",
    "monochrome",
    "cascade",
    "const_level",
    "ghost_tetromino_character",
    "ghost_tetromino_color",
    "top_border_character",
    "left_border_character",
    "bottom_border_character",
    "right_border_character",
    "tl_corner_character",
    "bl_corner_character",
    "br_corner_character",
    "tr_corner_character",
    "border_color",
    "block_character",
    "block_size",
    "mode",
    "move_left",
    "move_right",
    "rotate_clockwise",
    "rotate_anticlockwise",
    "soft_drop",
    "hard_drop",
    "hold",
    "background_color",
    "i_color",
    "j_color",
    "l_color",
    "s_color",
    "z_color",
    "t_color",
    "o_color"
];

const D_FPS: u64 = 60;
const D_BOARD_WIDTH: usize = 10;
const D_BOARD_HEIGHT: usize = 20;
const D_MODE: Mode = Mode::Modern;
const D_LEFT: KeyEvent = KeyEvent::Left;
const D_RIGHT: KeyEvent = KeyEvent::Right;
const D_ROT_CW: KeyEvent = KeyEvent::ShiftLeft;
const D_ROT_ACW: KeyEvent = KeyEvent::Up;
const D_SOFT_DROP: KeyEvent = KeyEvent::Down;
const D_HARD_DROP: KeyEvent = KeyEvent::Char(' ');
const D_HOLD: KeyEvent = KeyEvent::Char('c');
const D_GHOST_TETROMINO_CHARACTER: Option<char> = Some('⬜');
const D_GHOST_TETROMINO_COLOR: Option<Color> = Some(Color::Rgb {r: 240, g: 240, b: 240});
const D_CASCADE: bool = false;
const D_CONST_LEVEL: Option<usize> = None;
const D_MONOCHROME: Option<Color> = None;
const D_BORDER_COLOR: Color = Color::Rgb {r: 255, g: 255, b: 255};
const D_TOP_BORDER_CHARACTER: char = '═';
const D_TL_CORNER_CHARACTER: char = '╔';
const D_LEFT_BORDER_CHARACTER: char = '║';
const D_BL_CORNER_CHARACTER: char = '╚';
const D_BOTTOM_BORDER_CHARACTER: char = '═';
const D_BR_CORNER_CHARACTER: char = '╝';
const D_RIGHT_BORDER_CHARACTER: char = '║';
const D_TR_CORNER_CHARACTER: char = '╗';
const D_BACKGROUND_COLOR: Color = Color::Rgb {r: 0, g: 0, b: 0};
const D_BLOCK_CHARACTER: char = '⬛';
const D_BLOCK_SIZE: usize = 1;
const D_I_COLOR: Color = Color::Rgb {r: 0, g: 240, b: 240};
const D_J_COLOR: Color = Color::Rgb {r: 0, g: 0, b: 240};
const D_L_COLOR: Color = Color::Rgb {r: 240, g: 160, b: 0};
const D_S_COLOR: Color = Color::Rgb {r: 0, g: 240, b: 0};
const D_Z_COLOR: Color = Color::Rgb {r: 240, g: 0, b: 0};
const D_T_COLOR: Color = Color::Rgb {r: 160, g: 0, b: 240};
const D_O_COLOR: Color = Color::Rgb {r: 240, g: 240, b: 0};

const VALID_SETTINGS: &'static str = "\
Valid settings: fps, board_width, board_height, monochrome, cascade,\n\
const_level, ghost_tetromino, border_character, tl_corner_character,\n\
bl_corner_character, br_corner_character, tr_corner_character, border_color,\n\
block_character, block_size, mode, move_left, move_right, rotate_clockwise,\n\
rotate_anticlockwise, soft_drop, hard_drop, hold, background_color, i_color,\n\
j_color, l_color, s_color, z_color, t_color, o_color";

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Mode {
    Classic,
    Modern
}

pub enum ParseErrorKind {
    InvalidLineFormat,
    UnknownSetting,
    InvalidValue,
    DuplicateSetting,
    FailedParseValue,
    MissingValue
}

pub struct ParseError {
    kind: ParseErrorKind,
    line_no: usize,
    line: String,
    correction: Option<&'static str>
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, line_no: usize, line: &str, correction: Option<&'static str>)
        -> Self {
        ParseError {
            kind,
            line_no,
            line: line.to_owned(),
            correction
        }
    }
}

fn general_parse<T>(map: &mut HashMap<&str, (&str, usize, &str)>, key: &str, default: T,
    parser: fn(&str, usize, &str) -> Result<T, ParseError>) -> Result<T, ParseError> {
    if let Some((unparsed_setting, line_num, line)) = map.remove(key) {
        parser(unparsed_setting, line_num, line)
    } else {
        Ok(default)
    }
}

fn parse_u64(rhs: &str, line_num: usize, line: &str) -> Result<u64, ParseError> {
    let parsed_value = rhs.parse::<u64>().map_err(|| {
        ParseError::new(ParseErrorKind::FailedParseValue, line_num, line,
            Some("FPS setting takes an integer value."));
    })?;
    if parsed_value == 0 {
        Err(ParseError(ParseErrorKind::InvalidValue, line_num, line,
            Some("FPS value must be greater than 0.")))
    } else {
        Ok(parsed_value)
    }
}

fn parse_board_dimension(rhs: &str, line_num: usize, line: &str) -> Result<usize, ParseError> {
    let parsed_value = rhs.parse::<usize>().map_err(|| {
        ParseError::new(ParseErrorKind::FailedParseValue, line_num, line,
            Some("Board dimensions must be specified using integer values."))
    })?;
    if parsed_value == 0 {
        Err(ParseError(ParseErrorKind::InvalidValue, line_num, line,
            Some("Board dimensions must be greater than 0.")))
    } else {
        Ok(parsed_value)
    }
}

fn parse_mode(rhs: &str, line_num: usize, line: &str) -> Result<Mode, ParseError> {
    match rhs.to_ascii_lowercase().as_str() {
        "c" | "classic" => Ok(Mode::Classic),
        "m" | "modern" => Ok(Mode::Modern),
        _ => Err(ParseError::new(ParseErrorKind::InvalidValue, line_num, line,
            Some("Accepted game mode indicators: c, classic, m, modern.")))
    }
}

fn parse_keyevent(rhs: &str, line_num: usize, line: &str) -> Result<KeyEvent, ParseError> {
    match rhs.len() {
        1 => Ok(KeyEvent::Char(rhs.chars().next().unwrap())),
        _ => match rhs {
            "space" => Ok(KeyEvent::Char(' ')),
            "left" => Ok(KeyEvent::Left),
            "right" => Ok(KeyEvent::Right),
            "up" => Ok(KeyEvent::Up),
            "down" => Ok(KeyEvent::Down),
            "lshift" => Ok(KeyEvent::ShiftLeft),
            "rshift" => Ok(KeyEvent::ShiftRight),
            "lctrl" => Ok(KeyEvent::CtrlLeft),
            "rctrl" => Ok(KeyEvent::CtrlRight),
            "esc" => Ok(KeyEvent::Esc),
            _ => Err(ParseError::new(ParseErrorKind::InvalidValue, line_num, line,
                Some("Supported non-single-character values: 'space', 'left', 'right', 'up', \
                'down', 'lshift', 'rshift', 'lctrl', 'rctrl', and 'esc'.")))
        }
    }
}

fn parse_bool(rhs: &str, line_num: usize, line: &str) -> Result<bool, ParseError> {
    match rhs.to_ascii_lowercase().as_str() {
        "1" | "t" | "true" => Ok(true),
        "0" | "f" | "false" => Ok(false),
        _ => Err(ParseError::new(ParseErrorKind::InvalidValue, line_num, line,
            Some("Accepted boolean values: 1, t, true, 0, f, false")))
    }
}

fn parse_color(rhs: &str, line_num: usize, line: &str) -> Result<Color, ParseError> {
    let mut parts = rhs.split_whitespace();
    let color_type = parts.next().ok_or_else(|| ParseError::new(ParseErrorKind::MissingValue,
        line_num, line, Some("Missing color type.")))?;
    let color = parts.next().ok_or_else(|| ParseError::new(ParseErrorKind::MissingValue, line_num,
        line, Some("Missing color.")))?;
    match color_type.to_ascii_lowercase().as_str() {
        "rgb" => {
            let (r, g, b) = parse_rgb_triple(color, line_num, line)?;
            Ok(Color::Rgb {r, g, b})
        },
        "ansi" => {
            let c = color.parse::<u8>().map_err(|| ParseError::new(ParseErrorKind::FailedParseValue,
                line_num, line, Some("Failed to parse ANSI color value.")))?;
            Ok(Color::AnsiValue(c))
        },
        _ => Err(ParseError::new(ParseErrorKind::InvalidValue, line_num, line,
            Some("Accepted color formats are: rgb, ansi.")))
    }
}

fn parse_rgb_triple(s: &str, line_num: usize, line: &str) -> Result<(u8, u8, u8), ParseError> {
    let mut parts = s.split(',');
    let r = parts.next().ok_or_else(|| ParseError::new(ParseErrorKind::MissingValue, line_num, line,
        Some("Missing R value.")))?.parse::<u8>().map_err(|| ParseErrorKind::FailedParseValue,
        line_num, line, Some("Failed to parse R value."))?;
    let g = parts.next().ok_or_else(|| ParseError::new(ParseErrorKind::MissingValue, line_num, line,
        Some("Missing G value.")))?.parse::<u8>().map_err(|| ParseErrorKind::FailedParseValue,
        line_num, line, Some("Failed to parse G value."))?;
    let b = parts.next().ok_or_else(|| ParseError::new(ParseErrorKind::MissingValue, line_num, line,
        Some("Missing B value.")))?.parse::<u8>().map_err(|| ParseErrorKind::FailedParseValue,
        line_num, line, Some("Failed to parse B value."))?;
    Ok((r, g, b))
}

fn parse_char(rhs: &str, line_num: usize, line: &str) -> Result<char, ParseError> {
    if rhs.len() != 1 {
        Err(ParseError::new(ParseErrorKind::InvalidValue, line_num, line,
            Some("Expected a single character value.")))
    } else {
        Ok(rhs.chars().next().unwrap())
    }
}

fn parse_opt_char(rhs: &str, line_num: usize, line: &str) -> Result<Option<char>, ParseError> {
    if rhs.len() == 0 {
        Ok(None)
    } else {
        parse_char(rhs, line_num, line)
    }
}

pub struct GameConfig {
    // Required game settings
    fps: u64,
    board_width: usize,
    board_height: usize,
    mode: Mode,
    left: KeyEvent,
    right: KeyEvent,
    rot_cw: KeyEvent,
    rot_acw: KeyEvent,
    soft_drop: KeyEvent,
    hard_drop: KeyEvent,
    hold: KeyEvent,
    // Optional gameplay settings
    ghost_tetromino_character: Option<char>,
    ghost_tetromino_color: Option<Color>,
    cascade: bool,
    const_level: Option<usize>,
    // Optional game appearance setting
    monochrome: Option<Color>,
    // Optional board appearance settings
    border_color: Color,
    top_border_character: char,
    tl_corner_character: char,
    left_border_character: char,
    bl_corner_character: char,
    bottom_border_character: char,
    br_corner_character: char,
    right_border_character: char,
    tr_corner_character: char,
    background_color: Color,
    // Optional block appearance settings
    block_character: char,
    block_size: usize,
    i_color: Color,
    j_color: Color,
    l_color: Color,
    s_color: Color,
    z_color: Color,
    t_color: Color,
    o_color: Color
}

impl GameConfig {
    pub fn default() -> Self {
        GameConfig {
            fps: D_FPS,
            board_width: D_BOARD_WIDTH,
            board_height: D_BOARD_HEIGHT,
            mode: D_MODE,
            left: D_LEFT,
            right: D_RIGHT,
            rot_cw: D_ROT_CW,
            rot_acw: D_ROT_ACW,
            soft_drop: D_SOFT_DROP,
            hard_drop: D_HARD_DROP,
            hold: D_HOLD,
            ghost_tetromino_character: D_GHOST_TETROMINO_CHARACTER,
            ghost_tetromino_color: D_GHOST_TETROMINO_COLOR,
            cascade: D_CASCADE,
            const_level: D_CONST_LEVEL,
            monochrome: D_MONOCHROME,
            border_color: D_BORDER_COLOR,
            top_border_character: D_TOP_BORDER_CHARACTER,
            tl_corner_character: D_TL_CORNER_CHARACTER,
            left_border_character: D_LEFT_BORDER_CHARACTER,
            bl_corner_character: D_BL_CORNER_CHARACTER,
            bottom_border_character: D_BOTTOM_BORDER_CHARACTER,
            br_corner_character: D_BR_CORNER_CHARACTER,
            right_border_character: D_RIGHT_BORDER_CHARACTER,
            tr_corner_character: D_TR_CORNER_CHARACTER,
            background_color: D_BACKGROUND_COLOR,
            block_character: D_BLOCK_CHARACTER,
            block_size: D_BLOCK_SIZE,
            i_color: D_I_COLOR,
            j_color: D_J_COLOR,
            l_color: D_L_COLOR,
            s_color: D_S_COLOR,
            z_color: D_Z_COLOR,
            t_color: D_T_COLOR,
            o_color: D_O_COLOR
        }
    }

    fn parse(s: &str) /*-> Result<Self, ParseError>*/ {
        let mut settings = HashMap::with_capacity(31);
        for (num, line) in s.lines().enumerate() {
            if line.len() == 0 {
                continue;
            }
            if let Some('#') = line.chars().take(1).next() {
                continue;
            }
            let mut sections = line.split('=').trim();
            let lhs = sections.next()
                .ok_or_else(||
                    ParseError::new(ParseErrorKind::InvalidLineFormat, num, line, None)
                )?;
            if lhs.len() == 0 {
                return Err(ParseError::new(ParseErrorKind::InvalidLineFormat, num, line,
                    Some("There must be a setting name on the left side of the equals sign.")));
            }
            let rhs = sections.next()
                .ok_or_else(||
                    ParseError::new(ParseErrorKind::InvalidLineFormat, num, line, None)
                )?;
            if rhs.len() == 0 {
                return Err(ParseError::new(ParseErrorKind::InvalidLineFormat, num, line,
                    Some("There must be a value on the right side of the equals sign.")));
            }
            if CONFIG_OPTIONS.contains(lhs) {
                if settings.insert(lhs, (rhs, num, line)).is_some() {
                    return Err(ParseError::new(ParseErrorKind::DuplicateSetting, num, line, None));
                }
            } else {
                return Err({
                    ParseError::new(ParseErrorKind::UnknownSetting, num, line, Some(VALID_SETTINGS))
                });
            }
        }
        let fps = general_parse::<u64>(&mut settings, "fps", D_FPS, parse_u64)?;
        let board_width = general_parse::<usize>(&mut settings, "board_width", D_BOARD_WIDTH,
            parse_board_dimension)?;
        let board_height = general_parse::<usize>(&mut settings, "board_height", D_BOARD_HEIGHT,
            parse_board_dimension)?;
        let mode = general_parse::<Mode>(&mut settings, "mode", D_MODE, parse_mode)?;
        let left = general_parse::<KeyEvent>(&mut settings, "left", D_LEFT, parse_keyevent)?;
        let right = general_parse::<KeyEvent>(&mut settings, "right", D_RIGHT, parse_keyevent)?;
        let rot_cw = general_parse::<KeyEvent>(&mut settings, "rot_cw", D_ROT_CW, parse_keyevent)?;
        let rot_acw = general_parse::<KeyEvent>(&mut settings, "rot_acw", D_ROT_ACW,
            parse_keyevent)?;
        let soft_drop = general_parse::<KeyEvent>(&mut settings, "soft_drop", D_SOFT_DROP,
            parse_keyevent)?;
        let hard_drop = general_parse::<KeyEvent>(&mut settings, "hard_drop", D_HARD_DROP,
            parse_keyevent)?;
        let hold = general_parse::<KeyEvent>(&mut settings, "hold", D_HOLD, parse_keyevent)?;
        let ghost_tetromino_character = general_parse::<Option<char>>(&mut settings,
            "ghost_tetromino_character", D_GHOST_TETROMINO_CHARACTER, parse_opt_char)?;
        let ghost_tetromino_color = general_parse(&mut settings, "ghost_tetromino_color", ,)?;
        let cascade = general_parse(&mut settings, "cascade", ,)?;
        let const_level = general_parse(&mut settings, "const_level", ,)?;
        let monochrome = general_parse(&mut settings, "monochrome", ,)?;
        let border_color = general_parse(&mut settings, "border_color", ,)?;
        let top_border_character = general_parse::<char>(&mut settings, "top_border_character", ,)?;
        let tl_corner_character = general_parse::<char>(&mut settings, "tl_corner_character", ,)?;
        let left_border_character = general_parse::<char>(&mut settings, "left_border_character", ,)?;
        let bl_corner_character = general_parse::<char>(&mut settings, "bl_corner_character", ,)?;
        let bottom_border_character = general_parse::<char>(&mut settings, "bottom_border_character", ,)?;
        let br_corner_character = general_parse::<char>(&mut settings, "br_corner_character", ,)?;
        let right_border_character = general_parse::<char>(&mut settings, "right_border_character", ,)?;
        let tr_corner_character = general_parse::<char>(&mut settings, "tr_corner_character", ,)?;
        let background_color = general_parse(&mut settings, "background_color", ,)?;
        let block_character = general_parse::<char>(&mut settings, "block_character", ,)?;
        let block_size = general_parse(&mut settings, "block_size", ,)?;
        let i_color = general_parse(&mut settings, "i_color", ,)?;
        let j_color = general_parse(&mut settings, "j_color", ,)?;
        let l_color = general_parse(&mut settings, "l_color", ,)?;
        let s_color = general_parse(&mut settings, "s_color", ,)?;
        let z_color = general_parse(&mut settings, "z_color", ,)?;
        let t_color = general_parse(&mut settings, "t_color", ,)?;
        let o_color = general_parse(&mut settings, "o_color", ,)?;
    }
}

impl GameConfig {

    // Redundant code here since I wasn't sure how I was going to load/apply settings when I first
    // started writing it and I haven't gone back to delete it since making a decision.
    pub fn load_config(screen: &mut AlternateScreen<Stdout>) -> Option<Self> {
        let conf_file = File::open("../tui_tetris.conf");
        let gc = if let Ok(mut file) = conf_file {
            let mut file_string = String::new();
            if file.read_to_string(&mut file_string).is_err() {
                write!(screen, "Failed to read configuration file! Would you like to use the \
                    default config? [Y/n] ").unwrap();
                let mut response = String::new();
                stdin().read_line(&mut response).unwrap();
                while response != "" && response != "y" && response != "Y" && response != "n"
                    && response != "N" {
                    write!(screen, "Sorry, that's an invalid option. Would you like to use the \
                        default config? [Y/n] ").unwrap();
                }
                match response.as_str() {
                    "" | "y" | "Y" => Some(GameConfig::default()),
                    "n" | "N" => None,
                    _ => unreachable!()
                }
            } else {
                let lines = file_string.lines().collect::<Vec<&str>>();
                let mut set = [false; 21];
                let mut game_config = GameConfig::default();
                for (no, &line) in lines.iter().enumerate() {
                    if line == "" {
                        continue;
                    }
                    let lowercase_line = line.to_lowercase();
                    let option = lowercase_line.split(|c| c == ' ' || c == '=').next();
                    if option.is_none() {
                        write!(screen, "Uh oh, seems like your config file is malformed! Here's \
                            the problematic line: {}\n", line).unwrap();
                        return None;
                    }
                    let option = option.unwrap();
                    if let Some(i) = CONFIG_OPTIONS.iter().enumerate().filter(|&(j, _)| !set[j])
                        .position(|(_, &o)| o == option) {
                        match i {
                            0 => {
                                let fps = lowercase_line.chars().rev()
                                    .take_while(|&c| c != ' ' && c != '=')
                                    .collect::<String>();
                                let fps = fps.chars().rev().collect::<String>();
                                if let Ok(fps) = fps.parse::<u64>() {
                                    game_config.fps = fps;
                                    set[0] = true;
                                } else {
                                    write!(screen, "Whoops, looks like you specified an invalid \
                                        FPS on line {}: {}\n", no, line).unwrap();
                                    return None;
                                }
                            },
                            1 => {
                                let board_width = lowercase_line.chars().rev()
                                    .take_while(|&c| c != ' ' && c != '=')
                                    .collect::<String>();
                                let board_width = board_width.chars().rev().collect::<String>();
                                if let Ok(board_width) = board_width.parse::<usize>() {
                                    game_config.board_width = board_width;
                                    set[1] = true;
                                } else {
                                    write!(screen, "Whoops, looks like you specified an invalid \
                                        board width on line {}: {}\n", no, line).unwrap();
                                    return None;
                                }
                            },
                            2 => {
                                let board_height = lowercase_line.chars().rev()
                                    .take_while(|&c| c != ' ' && c != '=')
                                    .collect::<String>();
                                let board_height = board_height.chars().rev().collect::<String>();
                                if let Ok(board_height) = board_height.parse::<usize>() {
                                    game_config.board_height = board_height;
                                    set[1] = true;
                                } else {
                                    write!(screen, "Whoops, looks like you specified an invalid \
                                        board height on line {}: {}\n", no, line).unwrap();
                                    return None;
                                }
                            },
                            3 => {

                            },
                            4 => {

                            },
                            5 => {

                            },
                            6 => {

                            },
                            7 => {

                            },
                            8 => {

                            },
                            9 => {

                            },
                            10 => {

                            },
                            11 => {

                            },
                            12 => {

                            },
                            13 => {

                            },
                            14 => {

                            },
                            15 => {

                            },
                            16 => {

                            },
                            17 => {

                            },
                            18 => {

                            },
                            19 => {

                            },
                            20 => {

                            },
                            21 => {
                                if set[13] && game_config.mode == Mode::Classic {
                                    write!(screen, "Classic mode is selected but hold KeyEvent is \
                                        specified. Ignoring.\n").unwrap();
                                }
                                let hold_KeyEvent = lowercase_line.chars().rev()
                                    .take_while(|&c| c != ' ' && c != '=')
                                    .collect::<String>();
                                let hold_KeyEvent = hold_KeyEvent.chars().rev().collect::<String>();
                                if hold_KeyEvent.len() > 1 {
                                    match hold_KeyEvent.to_lowercase().as_str() {
                                        "up" => game_config.hold = KeyEvent::Up,
                                        "down" => game_config.hold = KeyEvent::Down,
                                        "left" => game_config.hold = KeyEvent::Left,
                                        "right" => game_config.hold = KeyEvent::Right,
                                        _ => {
                                            write!(screen, "{} is not an acceptable hold KeyEvent. Line \
                                                {}: {}\n", hold_KeyEvent, no, line).unwrap();
                                            return None;
                                        }
                                    }
                                } else {
                                    game_config.hold = KeyEvent::Char(hold_KeyEvent.chars().next().unwrap());

                                }
                            },
                            _ => unreachable!()
                        };
                    } else {
                        write!(screen, "Unrecognized option in config file: {}\n", line).unwrap();
                        return None;
                    }
                }
                Some(game_config)
            }
        } else {
            Some(GameConfig::default())
        };
        gc
    }
}

fn load_u64(s: &str) -> Result<Setting, LoadSettingError> {
    if setting_strs[last] == "" {
        println!("Found empty setting for FPS. Defaulting to: 60");
        Ok(Setting {
            field: CONFIG_OPTIONS[0].to_string(),
            value: SettingValue::u64(60)
        })
    } else if let Ok(num) = setting_strs[last].parse::<u64>() {
        Ok(Setting {
            field: CONFIG_OPTIONS[0].to_string(),
            value: SettingValue::u64(num)
        })
    } else {
        let err = format!("Found invalid value for fps: {}", setting_strs[last]);
        Err(LoadSettingError::ValueError(err))
    }
}

fn try_str_to_refstatic_color(s: &str) -> Option<Color> {
    if s == "black" {
        Some(&color::Black)
    } else if s == "blue" {
        Some(&color::Blue)
    } else if s == "cyan" {
        Some(&color::Cyan)
    } else if s == "green" {
        Some(&color::Green)
    } else if s == "lightblack" {
        Some(&color::LightBlack)
    } else if s == "lightblue" {
        Some(&color::LightBlue)
    } else if s == "lightcyan" {
        Some(&color::LightCyan)
    } else if s == "lightgreen" {
        Some(&color::LightGreen)
    } else if s == "lightmagenta" {
        Some(&color::LightMagenta)
    } else if s == "lightred" {
        Some(&color::LightRed)
    } else if s == "lightwhite" {
        Some(&color::LightWhite)
    } else if s == "lightyellow" {
        Some(&color::LightYellow)
    } else if s == "magenta" {
        Some(&color::Magenta)
    } else if s == "red" {
        Some(&color::Red)
    } else if s == "white" {
        Some(&color::White)
    } else if s == "yellow" {
        Some(&color::Yellow)
    } else {
        None
    }
}

const VALID_COLORS: [&str; 16] = ["black", "blue", "cyan", "green", "lightblack", "lightblue",
    "lightcyan", "lightgreen", "lightmagenta", "lightred", "lightwhite", "lightyellow", "magenta",
    "red", "white", "yellow"];