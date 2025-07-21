use ratatui::text::Line;

pub fn extend_lines(all_lines: &mut Vec<Line<'static>>, lines: Vec<Line<'static>>, end_with_empty_line: bool) {
	if lines.is_empty() {
		return;
	}
	all_lines.extend(lines);
	if end_with_empty_line {
		all_lines.push(Line::default());
	}
}
