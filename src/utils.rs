// Copyright ⓒ 2019-2020 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms.

use std::process;

/// Print `msg` to stderr and Exit with status 1
pub fn exit(msg: &str) -> ! {
	eprintln!("{}", msg);
	process::exit(1);
}
