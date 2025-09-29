
pub struct StaticList<K, V, const N: usize> {
	data: [(K, V); N],
}
impl<K, V, const N: usize> StaticList<K, V, N> {
	pub fn new(data: [(K, V); N]) -> Self {
		Self {
			data,
		}
	}

	pub fn search(&self, cb: impl Fn(&K) -> bool) -> Option<&V> {
		for i in &self.data {
			if cb(&i.0) {
				return Some(&i.1);
			}
		}
		None
	}
	
	pub fn search_mut(&mut self, cb: impl Fn(&K) -> bool) -> Option<&mut V> {
		for i in &mut self.data {
			if cb(&i.0) {
				return Some(&mut i.1);
			}
		}
		None
	}
}

