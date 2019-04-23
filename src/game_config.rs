use super::*;
use std::io::Read;
use std::hint::unreachable_unchecked;
use std::str::FromStr;

const CONFIG_OPTIONS: [&str; 31] = [
    "fps",
    "board_width",
    "board_height",
    "monochrome",
    "cascade",
    "const_level",
    "ghost_tetromino",
    "border_character",
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

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Mode {
    Classic,
    Modern
}

enum SettingValue {
    u64(u64),
    usize(usize),
    Option_refstatic_Color(Option<&'static Color>),
    Color(&'static Color),
    bool(bool),
    Option_usize(Option<usize>),
    char(char),
    Mode(Mode),
    Key(Key),
    Empty
}

enum LoadSettingError {
    ValueError(String),
    InvalidSetting(String)
}

struct Setting {
    field: String,
    value: SettingValue
}

impl Default for Setting {
    fn default() -> Self {
        Setting {
            field: "".to_string(),
            value: SettingValue::Empty
        }
    }
}

impl FromStr for Setting {
    type Err = LoadSettingError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let setting_strs = string.split(|c| c == ' ' || c == '=').collect::<Vec<&str>>();
        let last = setting_strs.len() - 1;
        let option = setting_strs[0].to_lowercase().as_str();
        let value = setting_strs[last].to_lowercase().as_str();
        if let Some(i) = CONFIG_OPTIONS.iter()
            .position(|&config_option| config_option == option) {
            match i {
                0 => { // fps
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
                },
                j @ 1 | 2 | 14 => { // board_width, board_height, block_size
                    if setting_strs[last] == "" {
                        let defaults = [0, 10, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
                        println!("Found empty setting for {}. Defaulting to: {}", CONFIG_OPTIONS[j], defaults[j]);
                        Ok(Setting {
                            field: CONFIG_OPTIONS[j].to_string(),
                            value: SettingValue::usize(defaults[j])
                        })
                    } else if let Ok(num) = setting_strs[last].parse::<usize>() {
                        Ok(Setting {
                            field: CONFIG_OPTIONS[j].to_string(),
                            value: SettingValue::usize(num)
                        })
                    } else {
                        let err = format!("Found invalid value for {}: {}", CONFIG_OPTIONS[j],
                            setting_strs[last]);
                        Err(LoadSettingError::ValueError(err))
                    }
                },
                3 => { //monochrome
                    if let Some(color) = try_str_to_refstatic_color(setting_strs[last]) {
                        Ok(Setting {
                            field: CONFIG_OPTIONS[3].to_string(),
                            value: Some(color)
                        })
                    } else if setting_strs.len() >= 3 && setting_strs.len() <= 5
                        && setting_strs[last - 1] == "ansi" {
                        if let Ok(gs) = setting_strs[last].parse::<u8>() {
                            if gs < 24 {
                                Ok(Setting {
                                    field: CONFIG_OPTIONS[3].to_string(),
                                    value: SettingValue::Option_refstatic_Color(
                                        Some(AnsiValue::grayscale(gs)))
                                })
                            } else {
                                let err = format!("Greyscale values must be less than 24!");
                                Err(LoadSettingError::ValueError(err))
                            }
                        } else {
                            let err = format!("Could not read specified greyscale value: {}",
                                setting_strs[last]);
                            Err(LoadSettingError::ValueError(err))
                        }
                    } else if setting_strs.len() >= 5 && setting_strs.len() <= 7
                        && (setting_strs[last - 3] == "ansi"
                        || setting_strs[last - 3] == "rgb") {
                        let mut rgb = [0; 3];
                        for i in 0..3 {
                            if let Ok(cv) = setting_strs[last - i].parse::<u8>() {
                                rgb[i] = cv;
                            } else {
                                let err = format!("Failed to read value ({}) in line: {}",
                                    setting_strs[last - i], string);
                                return Err(LoadSettingError::ValueError(err));
                            }
                        }
                        if setting_strs[last - 3] == "ansi" {
                            Ok(Setting {
                                field: CONFIG_OPTIONS[3].to_string(),
                                value: SettingValue::Option_refstatic_Color(Some(
                                    AnsiValue::rgb(rgb[2], rgb[1], rgb[0])))
                            })
                        } else {
                            Ok(Setting {
                                field: CONFIG_OPTIONS[3].to_string(),
                                value: SettingValue::Option_refstatic_Color(Some(
                                    Rgb(rgb[2], rgb[1], rgb[0])))
                            })
                        }
                    } else {
                        let err = format!("Invalid monochrome setting line: {}", string);
                        Err(LoadSettingError::ValueError(err))
                    }
                },
                j @ 4 | 6 => { // cascade, ghost tetromino
                    if setting_strs[last] == "" && j == 6 {
                        let default = if j == 4 {
                            false
                        } else {
                            true
                        };
                        println!("Found empty setting for ghost tetromino. Defaulting to: {}",
                            CONFIG_OPTIONS[j], default);
                        Ok(Setting {
                            field: CONFIG_OPTIONS[j].to_string(),
                            value: SettingValue::bool(default)
                        })
                    } else if value.as_str() == "t" || value.as_str() == "true"
                        || value.as_str() == "1" {
                        Ok(Setting {
                            field: CONFIG_OPTIONS[j].to_string(),
                            value: SettingValue::bool(true)
                        })
                    } else if value.as_str() == "f" || value.as_str() == "false"
                        || value.as_str() == "0" {
                        Ok(Setting {
                            field: CONFIG_OPTIONS[j].to_string(),
                            value: SettingValue::bool(false)
                        })
                    } else {
                        let err = format!("Invalid setting for {} on line: {}", CONFIG_OPTIONS[j],
                            string);
                        Err(LoadSettingError::ValueError(err))
                    }
                },
                5 => { // const_level
                    Ok(Setting {
                        field: CONFIG_OPTIONS[5].to_string(),
                        value: SettingValue::Option_usize(
                            if let Ok(lvl) = setting_strs[last].parse::<u64>() {
                                Some(lvl)
                            } else {
                                None
                            }
                        )
                    })
                },
                // border_character, corner characters, block character
                j @ 7 | 8 | 9 | 10 | 11 | 12 => {
                    if setting_strs[last] == "" {
                        println!("Found empty setting for {}. Defaulting to: █", CONFIG_OPTIONS[j]);
                        Ok(Setting {
                            field: CONFIG_OPTIONS[j].to_string(),
                            value: SettingValue::char('█')
                        })
                    }
                },
                8 => { // tl_corner_character

                },
                9 => { // bl_corner_character

                },
                10 => { // br_corner_character

                },
                11 => { // tr_corner_character

                },
                12 => { // border_color

                },
                13 => { // block_character

                },
                15 => { // mode

                },
                16 => { // move_left

                },
                17 => { // move_right

                },
                18 => { // rotate_clockwise

                },
                19 => { // rotate_anticlockwise

                },
                20 => { // soft_drop

                },
                21 => { // hard_drop

                },
                22 => { // hold

                },
                23 => { // background_color

                },
                24 => { // i piece color

                },
                25 => { // j piece color

                },
                26 => { // l piece color

                },
                27 => { // s piece color

                },
                28 => { // z piece color

                },
                29 => { // t piece color

                },
                30 => { // o piece color

                },
                _ => unreachable!()
            }
        } else {
            let err = format!("Invalid setting: {}", string);
            Err(LoadSettingError::InvalidSetting(err))
        }
    }
}

fn load_settings() -> Result<[Setting; 22], LoadSettingError> {
    let mut settings: [Setting; 22] = Default::default();
    if let Ok(mut conf_file) = File::open("../tui_tetris.conf") {
        let mut conf_file_contents = String::new();
        if let Ok(_) = conf_file.read_to_string(&mut conf_file_contents) {
            for (line_no, line) in conf_file_contents.lines().enumerate() {
                if line == "" {
                    continue;
                }
                println!("Read line: {:?}", line);
                let setting_string= line.chars().take_while(|&c| c != '=').collect::<String>();
                match
            }
        }
    }
    Ok(settings)
}

pub struct GameConfig {
    fps: u64,
    board_width: usize,
    board_height: usize,
    monochrome: Option<&'static Color>,
    cascade: bool,
    const_level: Option<usize>,
    ghost_tetromino: bool,
    border_character: char,
    tl_corner_character: char,
    bl_corner_character: char,
    br_corner_character: char,
    tr_corner_character: char,
    border_color: &'static Color,
    block_character: char,
    block_size: usize,
    mode: Mode,
    left: Key,
    right: Key,
    rot_cw: Key,
    rot_acw: Key,
    soft_drop: Key,
    hard_drop: Key,
    hold: Key,
    background_color: &'static Color,
    i_color: &'static Color,
    j_color: &'static Color,
    l_color: &'static Color,
    s_color: &'static Color,
    z_color: &'static Color,
    t_color: &'static Color,
    o_color: &'static Color
}

impl GameConfig {
    pub fn default() -> Self {
        GameConfig {
            fps: 60,
            board_width: 10,
            board_height: 20,
            monochrome: None,
            cascade: false,
            const_level: None,
            ghost_tetromino: true,
            border_character: '█',
            tl_corner_character: '█',
            bl_corner_character: '█',
            br_corner_character: '█',
            tr_corner_character: '█',
            border_color: &color::White,
            block_character: '■',
            block_size: 1,
            mode: Mode::Modern,
            left: Key::Left,
            right: Key::Right,
            rot_cw: Key::Char('z'),
            rot_acw: Key::Up,
            soft_drop: Key::Down,
            hard_drop: Key::Char(' '),
            hold: Key::Char('v'),
            background_color: &color::Black,
            i_color: &color::Cyan,
            j_color: &color::Blue,
            l_color: &color::LightRed,
            s_color: &color::Green,
            z_color: &color::Red,
            t_color: &color::Magenta,
            o_color: &color::Yellow,
        }
    }

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
                                    write!(screen, "Classic mode is selected but hold key is \
                                        specified. Ignoring.\n").unwrap();
                                }
                                let hold_key = lowercase_line.chars().rev()
                                    .take_while(|&c| c != ' ' && c != '=')
                                    .collect::<String>();
                                let hold_key = hold_key.chars().rev().collect::<String>();
                                if hold_key.len() > 1 {
                                    match hold_key.to_lowercase().as_str() {
                                        "up" => game_config.hold = Key::Up,
                                        "down" => game_config.hold = Key::Down,
                                        "left" => game_config.hold = Key::Left,
                                        "right" => game_config.hold = Key::Right,
                                        _ => {
                                            write!(screen, "{} is not an acceptable hold key. Line \
                                                {}: {}\n", hold_key, no, line).unwrap();
                                            return None;
                                        }
                                    }
                                } else {
                                    game_config.hold = Key::Char(hold_key.chars().next().unwrap());

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

fn try_str_to_refstatic_color(s: &str) -> Option<&'static Color> {
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