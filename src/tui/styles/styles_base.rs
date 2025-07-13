#![allow(unused)]
use ratatui::style::{Color, Style, Stylize};

// NOTE: Mac Default terminal does not support true colors
//       only 256 ANSI one.

// - Ratatui site: https://ratatui.rs/
//     - Paragraph / Text - https://ratatui.rs/recipes/widgets/paragraph/
//     - Block - https://ratatui.rs/recipes/widgets/block/
//
// - Terminal colors: https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit
// - Google materials colors: https://m2.material.io/design/color/the-color-system.html#tools-for-picking-colors

// NOTE: The naming & structure are still work in progress.
//       But at least as centralized as possible.

// -- Bkg Colors
pub const CLR_BKG_400: Color = Color::Indexed(236);
pub const CLR_BKG_500: Color = Color::Indexed(234);
pub const CLR_BKG_600: Color = Color::Indexed(232);

pub const CLR_BKG: Color = CLR_BKG_500;

pub const CLR_BKG_GRAY_DARK: Color = CLR_BKG_400;
pub const CLR_BKG_GRAY_DARKER: Color = CLR_BKG;
pub const CLR_BKG_BLACK: Color = Color::Indexed(0);

pub const CLR_BKG_PRIME: Color = Color::Indexed(12);

pub const CLR_BKG_ACT: Color = Color::Indexed(236);
pub const CLR_BKG_SEL: Color = Color::Indexed(15);

pub const CLR_BKG_WHITE: Color = Color::Indexed(255);

// -- Text Colors
pub const CLR_TXT_100: Color = Color::Indexed(255);
pub const CLR_TXT_400: Color = Color::Indexed(253);
pub const CLR_TXT_500: Color = Color::Indexed(252);
pub const CLR_TXT_600: Color = Color::Indexed(250);
pub const CLR_TXT_700: Color = Color::Indexed(244);
pub const CLR_TXT_800: Color = Color::Indexed(242);
pub const CLR_TXT: Color = CLR_TXT_500;

pub const CLR_TXT_WHITE: Color = Color::Indexed(15);

pub const CLR_TXT_BLUE: Color = Color::Indexed(45);
pub const CLR_TXT_YELLOW: Color = Color::Indexed(226);
pub const CLR_TXT_GREEN: Color = Color::Indexed(46);

pub const CLR_TXT_WAITING: Color = CLR_TXT_400;
pub const CLR_TXT_RUNNING: Color = CLR_TXT_BLUE;
pub const CLR_TXT_DONE: Color = CLR_TXT_GREEN;

// -- Text Styles
pub const STL_TXT: Style = Style::new();

pub const STL_TXT_ACT: Style = Style::new().fg(Color::White);
pub const STL_TXT_SEL: Style = Style::new().fg(CLR_TXT_BLUE);
pub const STL_TXT_ACTION: Style = Style::new().fg(CLR_TXT_BLUE);

// -- Nav Styles
pub const STL_NAV_ITEM_HIGHLIGHT: Style = Style::new().bg(CLR_BKG_SEL).fg(Color::Black);

// -- Field Styles
pub const STL_FIELD_LBL: Style = Style::new().bg(CLR_BKG).fg(CLR_TXT_700);
pub const STL_FIELD_VAL: Style = Style::new().fg(CLR_TXT_WHITE);

pub const STL_FIELD_LBL_DARK: Style = Style::new().bg(CLR_BKG_BLACK).fg(CLR_TXT_800);
pub const STL_FIELD_VAL_DARK: Style = Style::new().fg(CLR_TXT_600);

// -- Setion Styles
pub const STL_SECTION_MARKER: Style = Style::new().bg(CLR_BKG_400).fg(CLR_TXT_800);
pub const STL_SECTION_MARKER_INPUT: Style = Style::new().bg(CLR_BKG_400).fg(CLR_TXT_BLUE);
pub const STL_SECTION_MARKER_OUTPUT: Style = Style::new().bg(CLR_BKG_400).fg(CLR_TXT_GREEN);

// -- Tab Styles
pub const CLR_BKG_TAB_ACT: Color = CLR_BKG_GRAY_DARK;
pub fn stl_tab_dft() -> Style {
	Style::new().bg(CLR_BKG_GRAY_DARKER).fg(CLR_TXT_600)
}

pub fn stl_tab_act() -> Style {
	Style::new().bg(CLR_BKG_TAB_ACT).fg(CLR_TXT_WHITE).bold()
}
