/**
shared utilities
*/

pub fn gen_time() -> u64 {
	// todo: this probably sucks
	std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.unwrap_or(std::time::Duration::from_millis(0x6969696969696969))
		.as_millis()
		.wrapping_pow(7)
		.wrapping_pow(5) as u64
}

