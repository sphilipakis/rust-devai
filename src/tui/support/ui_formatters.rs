use crate::support::text::format_f64;

pub fn ui_fmt_cost(cost: Option<f64>) -> String {
	if let Some(cost) = cost {
		format!("${}", format_f64(cost))
	} else {
		"$...".to_string()
	}
}
