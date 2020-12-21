// Copyright ⓒ 2019-2020 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms.

use crate::utils;
use clap::{crate_version, App, Arg};

#[derive(Debug)]
pub struct Args {
	pub cmd: String,
	pub n_collapse: usize,
	pub n_lines: usize,
}

#[cfg_attr(tarpaulin, skip)]
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
			Arg::with_name("n_collapse")
				.short("n")
				.long("n-collapse")
				.value_name("N")
				.help("Directories with more than N children will initially be collapsed")
				.takes_value(true),
		)
		.arg(
			Arg::with_name("n_lines")
				.short("l")
				.long("n-lines")
				.value_name("N")
				.help("Max number of lines to use")
				.takes_value(true),
		)
		.get_matches();

	Args {
		cmd: matches.value_of("cmd").unwrap_or(default_cmd()).to_string(),
		n_collapse: parse_usize(matches.value_of("n_collapse"), "n_collapse", 0).unwrap_or(10),
		n_lines: parse_usize(matches.value_of("n_lines"), "n_lines", 3).unwrap_or(20),
	}
}

/// Get the default command to use. We naively assume that `fd` is the rust
/// fd-find binary.
#[cfg_attr(tarpaulin, skip)]
fn default_cmd() -> &'static str {
	if which::which("fd").is_ok() {
		"fd"
	} else {
		"find"
	}
}

fn parse_usize(given: Option<&str>, arg: &str, min: usize) -> Option<usize> {
	if let Some(value) = given {
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

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn parsing_usize_with_no_value() {
		assert_eq!(parse_usize(None, "lines", 3), None);
	}

	#[test]
	fn parsing_usize_with_ok_value() {
		assert_eq!(parse_usize(Some("5"), "lines", 3), Some(5));
	}
}
