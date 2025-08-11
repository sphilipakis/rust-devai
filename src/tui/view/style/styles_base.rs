#![allow(unused)] // Ok for this file since we want to eventualy build a hollistic style structure
use ratatui::style::{Color, Modifier, Style};

// NOTE: Mac Default terminal does not support true colors
//       only 256 ANSI one.

// - Ratatui site: https://ratatui.rs/
//     - Paragraph / Text - https://ratatui.rs/recipes/widgets/paragraph/
//     - Block - https://ratatui.rs/recipes/widgets/block/
//
// - Mouse / Widget hover - https://stackoverflow.com/questions/78263467/detecting-mouse-click-events-on-blocks-when-using-ratatui
//
// - Terminal colors: https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit
// - Google materials colors: https://m2.material.io/design/color/the-color-system.html#tools-for-picking-colors

// NOTE: The naming & structure are still work in progress.
//       But at least as centralized as possible.

// TIPS:
// - use `Style::new().add_modifier(Modifier::BOLD);` for const and bolc

// -- Primary Colors

pub const CLR_TXT_WHITE: Color = Color::Indexed(15);
pub const CLR_TXT_BLACK: Color = Color::Indexed(0);

// 12 is a good blue as well
pub const CLR_TXT_BLUE_DARK: Color = Color::Indexed(20);
pub const CLR_TXT_BLUE: Color = Color::Indexed(33);
pub const CLR_TXT_TEAL: Color = Color::Indexed(45);
pub const CLR_TXT_YELLOW: Color = Color::Indexed(226);
pub const CLR_TXT_GREEN: Color = Color::Indexed(46);

pub const CLR_TXT_RED: Color = Color::Indexed(196);

pub const CLR_BKG_WHITE: Color = Color::Indexed(255);
pub const CLR_BKG_BLUE: Color = Color::Indexed(20);
pub const CLR_BKG_TEAL: Color = Color::Indexed(39);
pub const CLR_BKG_YELLOW: Color = Color::Indexed(226);
pub const CLR_BKG_GREEN: Color = Color::Indexed(46);
pub const CLR_BKG_RED: Color = Color::Indexed(124);

pub const CLR_TXT_SKIP: Color = Color::Indexed(225);
pub const CLR_BKG_SKIP: Color = Color::Indexed(225);

// -- Bkg Colors
pub const CLR_BKG_300: Color = Color::Indexed(238);
pub const CLR_BKG_400: Color = Color::Indexed(236);
pub const CLR_BKG_500: Color = Color::Indexed(234);
pub const CLR_BKG_600: Color = Color::Indexed(232);

pub const CLR_BKG: Color = CLR_BKG_500;

pub const CLR_BKG_GRAY_DARK: Color = CLR_BKG_400;
pub const CLR_BKG_GRAY_DARKER: Color = CLR_BKG;
// Indexed(0) get styled by the terms
pub const CLR_BKG_BLACK: Color = CLR_BKG_600;

pub const CLR_BKG_PRIME: Color = Color::Indexed(12);

pub const CLR_BKG_ACT: Color = Color::Indexed(236);
pub const CLR_BKG_SEL: Color = Color::Indexed(15);

pub const CLR_BKG_RUNNING_WAIT: Color = CLR_BKG_GRAY_DARK;
pub const CLR_BKG_RUNNING_OTHER: Color = CLR_BKG_GRAY_DARK;
pub const CLR_BKG_RUNNING_AI: Color = CLR_BKG_YELLOW;
pub const CLR_BKG_RUNNING_SKIP: Color = CLR_BKG_SKIP;
pub const CLR_BKG_RUNNING_DONE: Color = CLR_BKG_GREEN;
pub const CLR_BKG_RUNNING_ERR: Color = CLR_BKG_RED;

// -- Text Colors
pub const CLR_TXT_100: Color = Color::Indexed(255);
pub const CLR_TXT_400: Color = Color::Indexed(253);
pub const CLR_TXT_500: Color = Color::Indexed(252);
pub const CLR_TXT_600: Color = Color::Indexed(250);
pub const CLR_TXT_650: Color = Color::Indexed(247);
pub const CLR_TXT_700: Color = Color::Indexed(244);
pub const CLR_TXT_800: Color = Color::Indexed(242);
pub const CLR_TXT_850: Color = Color::Indexed(240);
pub const CLR_TXT: Color = CLR_TXT_500;

pub const CLR_TXT_HOVER: Color = CLR_TXT_BLUE;
pub const CLR_TXT_HOVER_SHOW: Color = Color::Indexed(51);

pub const CLR_TXT_WAITING: Color = CLR_TXT_400;
pub const CLR_TXT_RUNNING: Color = CLR_TXT_TEAL;
pub const CLR_TXT_DONE: Color = CLR_TXT_GREEN;

// -- Text Styles
pub const STL_TXT: Style = Style::new();

pub const STL_TXT_ACT: Style = Style::new().fg(Color::White);
pub const STL_TXT_SEL: Style = Style::new().fg(CLR_TXT_TEAL);
pub const STL_TXT_ACTION: Style = Style::new().fg(CLR_TXT_BLUE);

// -- Nav Styles
pub const STL_NAV_ITEM_HIGHLIGHT: Style = Style::new().bg(CLR_BKG_SEL).fg(Color::Black);

// -- Field Styles
pub const STL_FIELD_LBL: Style = Style::new().bg(CLR_BKG).fg(CLR_TXT_850);
// For debug layout
// Other good choices: 24 (dark teal) 68 (dark blue), 187 (light yellow), 182 (pinkish), 146 (purple, not that good)
pub const STL_FIELD_VAL: Style = Style::new().fg(CLR_TXT_650);

pub const STL_FIELD_LBL_DARK: Style = Style::new().bg(CLR_BKG_BLACK).fg(CLR_TXT_800);
pub const STL_FIELD_VAL_DARK: Style = Style::new().bg(CLR_BKG_BLACK).fg(CLR_TXT_650);

// -- Section Styles
pub const STL_SECTION_MARKER: Style = Style::new().bg(CLR_BKG_400).fg(CLR_TXT_700);
pub const STL_SECTION_MARKER_INPUT: Style = Style::new().bg(CLR_BKG_400).fg(CLR_TXT_TEAL);
pub const STL_SECTION_MARKER_SKIP: Style = Style::new().bg(CLR_BKG_400).fg(CLR_TXT_SKIP);
pub const STL_SECTION_MARKER_OUTPUT: Style = Style::new().bg(CLR_BKG_400).fg(CLR_TXT_GREEN);
pub const STL_SECTION_MARKER_ERR: Style = Style::new().bg(CLR_BKG_400).fg(CLR_TXT_RED);
pub const STL_SECTION_MARKER_AI: Style = Style::new().bg(CLR_BKG_400).fg(CLR_TXT_YELLOW);
pub const STL_SECTION_TXT: Style = Style::new().fg(CLR_TXT_WHITE);

// -- Tab Styles
pub const CLR_BKG_TAB_ACT: Color = CLR_BKG_GRAY_DARK;

pub const STL_TAB_DEFAULT: Style = Style::new().bg(CLR_BKG_GRAY_DARKER).fg(CLR_TXT_600);
pub const STL_TAB_DEFAULT_HOVER: Style = Style::new().bg(CLR_BKG_GRAY_DARKER).fg(CLR_TXT_HOVER);
pub const STL_TAB_ACTIVE: Style = Style::new().bg(CLR_BKG_TAB_ACT).fg(CLR_TXT_400);
pub const STL_TAB_ACTIVE_HOVER: Style = Style::new().bg(CLR_BKG_TAB_ACT).fg(CLR_TXT_400); // same when active

// -- Section Styles
//pub const STL_PIN_MARKER: Style = Style::new().bg(Color::Indexed(23)).fg(CLR_TXT_600);

pub const STL_PIN_MARKER: Style = Style::new().bg(CLR_BKG_500).fg(Color::Indexed(122));
