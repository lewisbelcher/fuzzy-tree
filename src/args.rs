// Copyright ⓒ 2019-2020 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms.

use crate::utils;
use clap::{crate_version, App, Arg, ArgMatches};

#[derive(Debug)]
pub struct Args {
	pub cmd: String,
	pub collapse: usize,
	pub lines: usize,
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
			Arg::with_name("collapse")
				.short("n")
				.long("collapse")
				.value_name("N")
				.help("Directories with more than N children will initially be collapsed")
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
		cmd: matches.value_of("cmd").unwrap_or("fd").to_string(),
		collapse: parse_usize(&matches, "collapse", 0).unwrap_or(10),
		lines: parse_usize(&matches, "lines", 3).unwrap_or(20),
	}
}

fn parse_usize(matches: &ArgMatches, arg: &str, min: usize) -> Option<usize> {
	if let Some(value) = matches.value_of(arg) {
		if let Ok(v) = value.parse() {
			if v < min {
				utils::exit(&format!("option '--{}' must be >={}", arg, min))
			}
			Some(v)
		} else {
			utils::exit(&format!("invalid value for option '--{}': {}", arg, value));
		}
	} else {
		None
	}
}
