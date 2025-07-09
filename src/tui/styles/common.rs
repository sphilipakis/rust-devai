#![allow(unused)]
use ratatui::style::{Color, Style};

// - Ratatui site: https://ratatui.rs/
//     - Paragraph / Text - https://ratatui.rs/recipes/widgets/paragraph/
//     - Block - https://ratatui.rs/recipes/widgets/block/
//
// - Terminal colors: https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit

// -- Bkg Colors
pub const CLR_BKG_LIGHT: Color = Color::Indexed(240);
pub const CLR_BKG_GRAY: Color = Color::Indexed(238);
pub const CLR_BKG_GRAY_DARK: Color = Color::Indexed(236);
pub const CLR_BKG_GRAY_DARKER: Color = Color::Indexed(234);

pub const CLR_BKG_PRIME: Color = Color::Indexed(12);

pub const CLR_BKG_ACT: Color = Color::Indexed(236);
pub const CLR_BKG_SEL: Color = Color::Indexed(15);

pub const CLR_BKG_WHITE: Color = Color::Indexed(255);

// -- Txt Colors

pub const CLR_TXT_300: Color = Color::Indexed(250);
pub const CLR_TXT_400: Color = Color::Indexed(252);
pub const CLR_TXT: Color = Color::Indexed(255);

pub const CLR_TXT_WHITE: Color = Color::Indexed(15);

pub const CLR_TXT_SEL: Color = Color::Black;

pub const CLR_TXT_GREEN: Color = Color::Green;

// -- Styles
pub const STL_TXT: Style = Style::new();

pub const STL_TXT_LABEL: Style = Style::new().fg(CLR_TXT_300).bg(CLR_BKG_GRAY_DARK);
pub const STL_TXT_VALUE: Style = Style::new().fg(CLR_TXT_WHITE);

pub const STL_TXT_ACT: Style = Style::new().fg(Color::White);
pub const STL_TXT_SEL: Style = Style::new().fg(Color::Blue);
pub const STL_TXT_ACTION: Style = Style::new().fg(Color::Blue);
