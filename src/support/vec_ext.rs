pub trait VecExt<T> {
	fn extended<I: IntoIterator<Item = T>>(self, iter: I) -> Self;
}

impl<T> VecExt<T> for Vec<T> {
	fn extended<I: IntoIterator<Item = T>>(mut self, iter: I) -> Self {
		self.extend(iter);
		self
	}
}
