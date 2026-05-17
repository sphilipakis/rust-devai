#[derive(Debug, Clone, Copy)]
pub struct ModelPricing {
	pub name: &'static str,
	pub input_cached: Option<f64>,
	pub input_normal: f64,
	pub output_normal: f64,
	pub output_reasoning: Option<f64>,
}
