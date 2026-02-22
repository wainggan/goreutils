
pub trait Environment {}

pub mod base;
pub mod standalone;
pub mod draw;

#[macro_export]
macro_rules! libconstruct {
	($name:ident, $env:ident, |$left:ident, $right:ident| $block:block) => {
		pub fn $name<E: $env>($left: &mut dyn FnMut() -> Option<Value>, $right: &E) -> Value $block
	};
}

pub fn lib_standalone<E: standalone::EnvironmentStandalone>() -> crate::types::Library<'static, E> {
	&[
		("int", base::lib_fn_int),
		("flt", base::lib_fn_flt),
		("list", base::lib_fn_list),
		("cmp", base::lib_fn_cmp),
		("not", base::lib_fn_not),
		("neg", base::lib_fn_neg),
		("eq", base::lib_fn_eq),
		("add", base::lib_fn_add),
		("sub", base::lib_fn_sub),
		("mul", base::lib_fn_mul),
		("div", base::lib_fn_div),
		("pi", base::lib_fn_pi),
		("print", standalone::lib_fn_print),
	]
}

pub fn lib_draw<E: draw::EnvironmentDraw>() -> crate::types::Library<'static, E> {
	&[
		("int", base::lib_fn_int),
		("flt", base::lib_fn_flt),
		("list", base::lib_fn_list),
		("cmp", base::lib_fn_cmp),
		("not", base::lib_fn_not),
		("neg", base::lib_fn_neg),
		("eq", base::lib_fn_eq),
		("add", base::lib_fn_add),
		("sub", base::lib_fn_sub),
		("mul", base::lib_fn_mul),
		("div", base::lib_fn_div),
		("pi", base::lib_fn_pi),
		("uv_x", draw::lib_fn_uv_x),
		("uv_y", draw::lib_fn_uv_y),
		("px_x", draw::lib_fn_px_x),
		("px_y", draw::lib_fn_px_y),
		("width", draw::lib_fn_width),
		("height", draw::lib_fn_height),
	]
}

