#![allow(unused)]
use ratatui::style::{Color, Style};

// -- Bkg Colors
pub const CLR_BKG_GRAY_DARK: Color = Color::Indexed(236);
pub const CLR_BKG_GRAY_DARKER: Color = Color::Indexed(234);

pub const CLR_BKG_PRIME: Color = Color::Indexed(12);

pub const CLR_BKG_ACT: Color = Color::Indexed(236);
pub const CLR_BKG_SEL: Color = Color::Indexed(15);

// -- Txt Colors

pub const CLR_TXT_400: Color = Color::Indexed(252);
pub const CLR_TXT: Color = Color::Indexed(255);

pub const CLR_TXT_SEL: Color = Color::Black;

pub const CLR_TXT_GREEN: Color = Color::Green;

// -- Styles
pub const STL_TXT: Style = Style::new();
pub const STL_TXT_ACT: Style = Style::new().fg(Color::White);
pub const STL_TXT_SEL: Style = Style::new().fg(Color::Blue);
