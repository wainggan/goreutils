
pub struct XorShift64(u64);
impl XorShift64 {
	pub fn new(seed: u64) -> Self {
		Self(seed)
	}
	pub fn new_entropy() -> Option<Self> {
		getrandom::u64().ok().map(|x| XorShift64(x))
	}
	pub fn new_time() -> Self {
		let a = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or(std::time::Duration::from_millis(0x6969696969696969))
			.as_millis()
			.wrapping_pow(7)
			.wrapping_pow(5) as u64;
		Self(a)
	}
	pub fn next(&mut self) -> f64 {
		let mut x = self.0;
		x ^= x << 13;
		x ^= x >> 7;
		x ^= x << 17;
		self.0 = x;
		return x as f64 / u64::MAX as f64;
	}
}

