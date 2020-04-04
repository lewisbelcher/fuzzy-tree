use clap::{crate_version, App, Arg};
use std::process;

#[derive(Debug)]
pub struct Args {
	pub lines: usize,
	pub cmd: String,
}

pub fn collect() -> Args {
	let matches = App::new("Fuzzy Tree")
		.version(crate_version!())
		.author("Lewis B. <gitlab.io/lewisbelcher>")
		.about("A filesystem fuzzy finder which displays results as an interactive tree.")
		.arg(
			Arg::with_name("cmd")
				.short("c")
				.long("cmd")
				.value_name("CMD")
				.help("Command to use for finding files")
				.takes_value(true),
		)
		.arg(
			Arg::with_name("lines")
				.short("l")
				.long("lines")
				.value_name("N")
				.help("Max number of lines to use")
				.takes_value(true),
		)
		.get_matches();

	Args {
		cmd: parse_cmd(matches.value_of("cmd")),
		lines: parse_lines(matches.value_of("lines")),
	}
}

fn parse_cmd(value: Option<&str>) -> String {
	value.unwrap_or("fd").to_string()
}

fn parse_lines(value: Option<&str>) -> usize {
	let value = value.unwrap_or("20");
	match value.parse() {
		Ok(v) => {
			if v < 3 {
				eprintln!("option '--lines' must be >=3");
				process::exit(1);
			}
			v
		}
		Err(_) => {
			eprintln!("invalid integer for option '--lines': {}", value);
			process::exit(1);
		}
	}
}
