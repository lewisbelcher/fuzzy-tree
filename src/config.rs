use std::env;

pub fn debug() -> bool {
	// lazy_static! {
	// 	static ref DEBUG: bool = env::var("DEBUG").is_ok();
	// }
	// TODO: Use lazy static
	return env::var("DEBUG").is_ok();
}
