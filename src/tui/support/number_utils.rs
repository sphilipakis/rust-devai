/// Will return an idx within 0 and len - 1 (or 0 if len is 0)
/// - `len` - the length of the aray
pub fn clamp_idx_in_len(idx: usize, len: usize) -> usize {
	if len == 0 {
		return 0; // No items, return 0
	}
	let max_idx = (len - 1).max(0);
	idx.clamp(0, max_idx) // cannot fail
}

/// Will return an idx within 0 and len - 1 (or 0 if len is 0)
/// - `len` - the length of the aray
pub fn offset_and_clamp_option_idx_in_len(idx: &Option<i32>, offset: i32, len: usize) -> Option<i32> {
	match (len, &idx) {
		(0, _) => None,
		(len, Some(idx_val)) => Some((*idx_val + offset).max(0).min(len as i32 - 1)),
		(_, None) => Some(0),
	}
}

pub fn num_pad_for_len(idx: i64, max_num: usize) -> String {
	let width = if max_num == 0 {
		1
	} else {
		(max_num as f64).log10().floor() as usize + 1
	};
	format!("{:0width$}", idx + 1, width = width)
}
