
fn main() {
	unsafe {
		#[allow(invalid_null_arguments)]
		std::ptr::write(std::ptr::null_mut(), 0);
	}
}

