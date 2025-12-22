use lazy_regex::regex;

pub struct TextSeg<'a> {
	pub text: String,
	pub file_path: Option<&'a str>,
}

pub fn segment_line_path(line: &str) -> Vec<TextSeg<'_>> {
	// Ends with 2 to 5 character extension, and includes '/'
	let re = regex!(r#"[a-zA-Z0-9_@\-\./]*/[a-zA-Z0-9_@\-\.]+\.[a-zA-Z0-9]{2,5}"#);
	let mut segments = Vec::new();
	let mut last_idx = 0;

	for m in re.find_iter(line) {
		let start = m.start();
		let end = m.end();
		let text = &line[start..end];

		// Must contain '/' to be considered a path in this context
		if text.contains('/') {
			if start > last_idx {
				segments.push(TextSeg {
					text: line[last_idx..start].to_string(),
					file_path: None,
				});
			}
			segments.push(TextSeg {
				text: text.to_string(),
				file_path: Some(text),
			});
			last_idx = end;
		}
	}

	if last_idx < line.len() {
		segments.push(TextSeg {
			text: line[last_idx..].to_string(),
			file_path: None,
		});
	}

	if segments.is_empty() && !line.is_empty() {
		segments.push(TextSeg {
			text: line.to_string(),
			file_path: None,
		});
	}

	segments
}
