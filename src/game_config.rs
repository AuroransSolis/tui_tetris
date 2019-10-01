use super::*;
use std::io::Read;
use std::hint::unreachable_unchecked;
use std::str::FromStr;
use std::collections::HashMap;
use std::ops::{Range, RangeFrom, RangeTo, RangeBounds};
use crossterm::{Color, InputEvent, KeyEvent};

type Settings = Settings;

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
const D_HARD_DROP: KeyEvent = Some(KeyEvent::Char(' '));
const D_HOLD: KeyEvent = Some(KeyEvent::Char('c'));
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

fn general_parse<T>(map: &mut Settings, key: &str, default: T,
    parser: fn(&str, usize, &str) -> Result<T, ParseError>) -> Result<T, ParseError> {
    if let Some(&(unparsed_setting, line_num, line)) = map.get(key) {
        parser(unparsed_setting, line_num, line)
    } else {
        Ok(default)
    }
}

fn opt_general_parse<T>(map: &mut Settings, key: &str, default: Option<T>,
    parser: fn(&str, usize, &str) -> Result<T, ParseError>) -> Result<Option<T>, ParseError> {
    if let Some(&(rhs, line_num, line)) = map.get(key) {
        if rhs.to_ascii_lowercase().as_str() == "none" {
            Ok(None)
        } else {
            Ok(Some(parser(rhs, line_num, line)?))
        }
    } else {
        default
    }
}

fn parse_num_range<T: PartialOrd + FromStr, R: RangeBounds>(map: &mut Settings, key: &str,
    default: T, range: R, fp_message: &'static str, oor_message: &'static str)
    -> Result<T, ParseError> {
    if let Some(&(rhs, line_num, line)) = map.get(key) {
        let parsed = rhs.parse::<T>().map_err(|| ParseError::new(ParseErrorKind::FailedParseValue,
            line_num, line, Some(fp_message)))?;
        if range.contains(&parsed) {
            Ok(parsed)
        } else {
            Err(ParseError::new(ParseErrorKind::InvalidValue, line_num, line, Some(oor_message)))
        }
    } else {
        Ok(default)
    }
}

fn opt_parse_num_range<T: PartialOrd + FromStr, R: RangeBounds>(map: &mut Settings, key: &str,
    default: Option<T>, range: R, fp_message: &'static str, oor_message: &'static str)
    -> Result<Option<T>, ParseError> {
    if let Some(&(rhs, line_num, line)) = map.get(key) {
        if rhs.to_ascii_lowercase().as_str == "none" {
            Ok(None)
        } else {
            let parsed = rhs.parse::<T>()
                .map_err(|| ParseError::new(ParseErrorKind::FailedParseValue, line_num, line,
                    Some(fp_message)))?;
            if range.contains(&parsed) {
                Ok(Some(parsed))
            } else {
                Err(ParseError::new(ParseErrorKind::InvalidValue, line_num, line,
                    Some(oor_message)))
            }
        }
    } else {
        Ok(default)
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

fn parse_bool(rhs: &str, line_num: usize, line: &str) -> Result<bool, ParseError> {
    match rhs.to_ascii_lowercase().as_str() {
        "1" | "t" | "true" => Ok(true),
        "0" | "f" | "false" => Ok(false),
        _ => Err(ParseError::new(ParseErrorKind::InvalidValue, line_num, line,
            Some("Accepted boolean values: 1, t, true, 0, f, false")))
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
    hard_drop: Option<KeyEvent>,
    hold: Option<KeyEvent>,
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

    fn parse(s: &str) -> Result<Self, ParseError> {
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
        let fps = parse_num_range::<u64, RangeFrom<u64>>(&mut settings, "fps", D_FPS, (1..),
            "Failed to parse FPS value.", "FPS value is not greater than or equal to 1.")?;
        let board_width = parse_num_range::<usize, RangeFrom<usize>>(&mut settings, "board_width",
            D_BOARD_WIDTH, (1..), "Failed to parse board width value.",
            "Board width value is not greater than or equal to 1.")?;
        let board_height = parse_num_range::<usize, RangeFrom<usize>>(&mut settings, "board_height",
            D_BOARD_HEIGHT, (1..), "Failed to parse board height value.",
            "Board height value is not greater than or equal to 1.")?;
        let mode = general_parse::<Mode>(&mut settings, "mode", D_MODE, parse_mode)?;
        let left = general_parse::<KeyEvent>(&mut settings, "left", D_LEFT, parse_keyevent)?;
        let right = general_parse::<KeyEvent>(&mut settings, "right", D_RIGHT, parse_keyevent)?;
        let rot_cw = general_parse::<KeyEvent>(&mut settings, "rot_cw", D_ROT_CW, parse_keyevent)?;
        let rot_acw = general_parse::<KeyEvent>(&mut settings, "rot_acw", D_ROT_ACW,
            parse_keyevent)?;
        let soft_drop = general_parse::<KeyEvent>(&mut settings, "soft_drop", D_SOFT_DROP,
            parse_keyevent)?;
        let mut hard_drop = general_parse::<KeyEvent>(&mut settings, "hard_drop", D_HARD_DROP,
            parse_keyevent)?;
        let mut hold = opt_general_parse::<KeyEvent>(&mut settings, "hold", D_HOLD, parse_keyevent)?;
        let mut ghost_tetromino_character = opt_general_parse::<char>(&mut settings,
            "ghost_tetromino_character", D_GHOST_TETROMINO_CHARACTER, parse_char)?;
        let mut ghost_tetromino_color = opt_general_parse::<Color>(&mut settings,
            "ghost_tetromino_color", D_GHOST_TETROMINO_COLOR, parse_color)?;
        let cascade = general_parse::<bool>(&mut settings, "cascade", D_CASCADE, parse_bool)?;
        let const_level = opt_parse_num_range::<usize, RangeFrom<usize>>(&mut settings,
            "const_level", D_CONST_LEVEL, (1..), "Failed to parse constant level value.",
            "Level value was not greater than or equal to 1.")?;
        let mut monochrome = opt_general_parse::<Color>(&mut settings, "monochrome", D_MONOCHROME,
            parse_color)?;
        let border_color = general_parse::<Color>(&mut settings, "border_color", D_BORDER_COLOR,
            parse_color)?;
        let top_border_character = general_parse::<char>(&mut settings, "top_border_character",
            D_TOP_BORDER_CHARACTER, parse_char)?;
        let tl_corner_character = general_parse::<char>(&mut settings, "tl_corner_character",
            D_TL_CORNER_CHARACTER, parse_char)?;
        let left_border_character = general_parse::<char>(&mut settings, "left_border_character",
            D_LEFT_BORDER_CHARACTER, parse_char)?;
        let bl_corner_character = general_parse::<char>(&mut settings, "bl_corner_character",
            D_BL_CORNER_CHARACTER, parse_char)?;
        let bottom_border_character = general_parse::<char>(&mut settings,
            "bottom_border_character", D_BOTTOM_BORDER_CHARACTER, parse_char)?;
        let br_corner_character = general_parse::<char>(&mut settings, "br_corner_character",
            D_BR_CORNER_CHARACTER, parse_char)?;
        let right_border_character = general_parse::<char>(&mut settings, "right_border_character",
            D_RIGHT_BORDER_CHARACTER, parse_char)?;
        let tr_corner_character = general_parse::<char>(&mut settings, "tr_corner_character",
            D_TR_CORNER_CHARACTER, parse_char)?;
        let background_color = general_parse::<Color>(&mut settings, "background_color",
            D_BACKGROUND_COLOR, parse_color)?;
        let block_character = general_parse::<char>(&mut settings, "block_character",
            D_BLOCK_CHARACTER, parse_char)?;
        let block_size = parse_num_range::<usize, RangeFrom<usize>>(&mut settings, "block_size",
            D_BLOCK_SIZE, (1..), "Failed to parse block size value.",
            "Block size must be greater than or equal to 1.")?;
        let mut i_color = general_parse(&mut settings, "i_color", D_I_COLOR, parse_color)?;
        let mut j_color = general_parse(&mut settings, "j_color", D_J_COLOR, parse_color)?;
        let mut l_color = general_parse(&mut settings, "l_color", D_L_COLOR, parse_color)?;
        let mut s_color = general_parse(&mut settings, "s_color", D_S_COLOR, parse_color)?;
        let mut z_color = general_parse(&mut settings, "z_color", D_Z_COLOR, parse_color)?;
        let mut t_color = general_parse(&mut settings, "t_color", D_T_COLOR, parse_color)?;
        let mut o_color = general_parse(&mut settings, "o_color", D_O_COLOR, parse_color)?;
        if board_width <= block_size || board_height <= block_size {
            let (line_num, line) = if let Some(&(_, line_num, line)) = settings.get("block_size") {
                (line_num, line)
            } else if let Some(&(_, line_num, line)) = settings.get("board_height") {
                (line_num, line)
            } else if let Some(&(_, line_num, line)) = settings.get("board_width") {
                (line_num, line)
            } else {
                unreachable!()
            };
            return Err(ParseError::new(ParseErrorKind::InvalidValue, line_num, line,
                Some("Board dimensions must be greater than or equal to block size.")));
        } else if monochrome.is_some() {
            i_color = monochrome.unwrap();
            j_color = monochrome.unwrap();
            l_color = monochrome.unwrap();
            s_color = monochrome.unwrap();
            z_color = monochrome.unwrap();
            t_color = monochrome.unwrap();
            o_color = monochrome.unwrap();
        } else {
            if mode == Mode::Classic {
                hard_drop = None;
                hold = None;
                ghost_tetromino_character = None;
                ghost_tetromino_color = None;
            }
        }
        Ok(GameConfig {
            fps,
            board_width,
            board_height,
            mode,
            left,
            right,
            rot_cw,
            rot_acw,
            soft_drop,
            hard_drop,
            hold,
            ghost_tetromino_character,
            ghost_tetromino_color,
            cascade,
            const_level,
            monochrome,
            border_color,
            top_border_character,
            tl_corner_character,
            left_border_character,
            bl_corner_character,
            bottom_border_character,
            br_corner_character,
            right_border_character,
            tr_corner_character,
            background_color,
            block_character,
            block_size,
            i_color,
            j_color,
            l_color,
            s_color,
            z_color,
            t_color,
            o_color
        })
    }
}