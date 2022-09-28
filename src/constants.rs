pub const TILE_SIZE: f32 = 32.;
pub const TILE_COUNT: f32 = 17.;
pub const WINDOW_SIZE: f32 = TILE_SIZE * TILE_COUNT;

pub const BOTTOM_BUTTON_Y: f32 = (-WINDOW_SIZE / 2.0) + TILE_SIZE;
pub const UI_BUTTON_WIDTH: f32 = 160.0;
pub const UI_BUTTON_HEIGHT: f32 = 48.0;

pub const COMPUTER_BUFFER_TIMER_SECS: f32 = 0.75;
pub const COMPUTER_MOVE_TIMER_SECS: f32 = 1.5;

pub const START_INDEX: usize = 0;
pub const CENTER_INDEX: usize = 53;
pub const FIRST_HOME_INDEX: usize = 48;
pub const LAST_HOME_INDEX: usize = 52;
pub const CENTER_ENTRANCE_INDEXES: [usize; 3] = [5, 17, 29];
pub const CENTER_EXIT_INDEX: usize = 41;
pub const POWER_UP_INDEXES: [usize; 5] = [3, 15, 27, 39, 53];

/// Main board cell indexes - rotate clockwise for each color
///
///                10 11 12
///                 9 -- 13
///                 8 -- 14
///                 7 -- 15
/// red             6 -- 16          green
///  0  1  2  3  4  5 -- 17 18 19 20 21 22
/// 47 48 49 50 51 52 53 -- -- -- -- -- 23
/// 46 45 44 43 42 41 -- 29 28 27 26 25 24
/// yellow         40 -- 30           blue
///                39 -- 31
///                38 -- 32
///                37 -- 33
///                36 35 34
///
pub const BOARD: [(i32, i32); 54] = [
    (-6, 1), // 0: start
    (-5, 1),
    (-4, 1),
    (-3, 1),
    (-2, 1),

    (-1, 1), // 5: shortcut entrance

    (-1, 2),
    (-1, 3),
    (-1, 4),
    (-1, 5),
    (-1, 6),

    (0, 6),

    (1, 6),
    (1, 5),
    (1, 4),
    (1, 3),
    (1, 2),

    (1, 1), // 17: shortcut entrance

    (2, 1),
    (3, 1),
    (4, 1),
    (5, 1),
    (6, 1),

    (6, 0),

    (6, -1),
    (5, -1),
    (4, -1),
    (3, -1),
    (2, -1),

    (1, -1), // 29: shortcut entrance

    (1, -2),
    (1, -3),
    (1, -4),
    (1, -5),
    (1, -6),

    (0, -6),

    (-1, -6),
    (-1, -5),
    (-1, -4),
    (-1, -3),
    (-1, -2),

    (-1, -1),

    (-2, -1),
    (-3, -1),
    (-4, -1),
    (-5, -1),
    (-6, -1),

    (-6, 0), // 47: home entrance

    // 48-52: home
    (-5, 0),
    (-4, 0),
    (-3, 0),
    (-2, 0),
    (-1, 0),

    (0, 0), // 53: center
];
