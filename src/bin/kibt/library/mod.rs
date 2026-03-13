
pub trait Environment {}

pub mod base;
pub mod standalone;
pub mod draw;

#[macro_export]
macro_rules! lib_construct {
	($name:ident, $env:ident, |$left:ident, $right:ident| $block:block) => {
		pub fn $name<E: $env>($left: &mut dyn FnMut() -> Option<Value>, $right: &E) -> Value $block
	};
}

pub fn lib_standalone<'a, E: standalone::EnvironmentStandalone>() -> crate::types::Library<'a, E> {
	&[
		("int", base::lib_fn_int),
		("flt", base::lib_fn_flt),
		("bool", base::lib_fn_bool),
		("list", base::lib_fn_list),
		("lt", base::lib_fn_lt),
		("gt", base::lib_fn_gt),
		("lte", base::lib_fn_lte),
		("gt", base::lib_fn_gte),
		("and", base::lib_fn_and),
		("or", base::lib_fn_or),
		("xor", base::lib_fn_xor),
		("not", base::lib_fn_not),
		("neg", base::lib_fn_neg),
		("eq", base::lib_fn_eq),
		("ne", base::lib_fn_ne),
		("add", base::lib_fn_add),
		("sub", base::lib_fn_sub),
		("mul", base::lib_fn_mul),
		("div", base::lib_fn_div),
		("rem", base::lib_fn_rem),
		("pow", base::lib_fn_pow),
		("abs", base::lib_fn_abs),
		("clamp", base::lib_fn_clamp),
		("sin", base::lib_fn_sin),
		("asin", base::lib_fn_asin),
		("cos", base::lib_fn_cos),
		("acos", base::lib_fn_acos),
		("tan", base::lib_fn_tan),
		("atan", base::lib_fn_atan),
		("atan2", base::lib_fn_atan2),
		("cat", base::lib_fn_cat),
		("pi", base::lib_fn_pi),

		("print", standalone::lib_fn_print),
	]
}

pub fn lib_draw<'a, E: draw::EnvironmentDraw>() -> crate::types::Library<'a, E> {
	&[
		("int", base::lib_fn_int),
		("flt", base::lib_fn_flt),
		("bool", base::lib_fn_bool),
		("list", base::lib_fn_list),
		("lt", base::lib_fn_lt),
		("gt", base::lib_fn_gt),
		("lte", base::lib_fn_lte),
		("gt", base::lib_fn_gte),
		("not", base::lib_fn_not),
		("neg", base::lib_fn_neg),
		("eq", base::lib_fn_eq),
		("add", base::lib_fn_add),
		("sub", base::lib_fn_sub),
		("mul", base::lib_fn_mul),
		("div", base::lib_fn_div),
		("rem", base::lib_fn_rem),
		("pow", base::lib_fn_pow),
		("abs", base::lib_fn_abs),
		("clamp", base::lib_fn_clamp),
		("sin", base::lib_fn_sin),
		("asin", base::lib_fn_asin),
		("cos", base::lib_fn_cos),
		("acos", base::lib_fn_acos),
		("tan", base::lib_fn_tan),
		("atan", base::lib_fn_atan),
		("atan2", base::lib_fn_atan2),
		("cat", base::lib_fn_cat),
		("pi", base::lib_fn_pi),
	
		("uv_x", draw::lib_fn_uv_x),
		("uv_y", draw::lib_fn_uv_y),
		("px_x", draw::lib_fn_px_x),
		("px_y", draw::lib_fn_px_y),
		("width", draw::lib_fn_width),
		("height", draw::lib_fn_height),
		("r", draw::lib_fn_sample_r),
		("g", draw::lib_fn_sample_g),
		("b", draw::lib_fn_sample_b),
	]
}

