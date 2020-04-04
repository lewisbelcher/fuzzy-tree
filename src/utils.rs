use std::process;

/// Print `msg` to stderr and Exit with status 1
pub fn exit(msg: &str) -> ! {
	eprintln!("{}", msg);
	process::exit(1);
}
