// Copyright ⓒ 2019-2020 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms.

#[macro_use]
mod args;
#[macro_use]
mod path;
mod tree;
mod tui;
mod utils;

#[macro_use]
extern crate log;

use log::Level;
use std::io;
use std::mem;
use std::process::{self, Command};
use termion::color;
use termion::event::Key;

fn main() -> Result<(), io::Error> {
	// env_logger::init();

	let cliargs = args::collect();
	debug!("{:?}", cliargs);

	let stdout = run_cmd(&cliargs.cmd).unwrap_or_else(|e| {
		utils::exit(&format!(
			"Failed to execute command `{}`: {}",
			&cliargs.cmd,
			e.to_string()
		))
	});
	run_loop(stdout, cliargs.n_collapse, cliargs.n_lines)
}

fn run_cmd(cmd: &str) -> Result<Vec<u8>, io::Error> {
	let (cmd, args) = {
		let mut split: Vec<&str> = cmd.split(' ').collect();
		(split.remove(0), split)
	};
	Ok(Command::new(cmd).args(&args).output()?.stdout)
}

fn run_loop(content: Vec<u8>, n_collapse: usize, n_lines: usize) -> Result<(), io::Error> {
	let mut tree = tree::Tree::from_stdout(content)?;
	if n_collapse > 0 {
		tree.collapse_over(n_collapse)
	}
	let lines = tree.as_lines();
	let prompt = format!("{}> {}", color::Fg(color::Blue), color::Fg(color::Reset));
	let mut ui = tui::Tui::new(prompt, n_lines, lines.len())?;

	ui.render(tree.info_line(), lines)?;

	for c in tui::iter_keys() {
		match c? {
			Key::Esc => break,
			Key::Char(c) => {
				if c == '\t' {
					tree.flip_selected(ui.index());
					ui.move_down();
				} else if c == '`' {
					tree.flip_open(ui.index());
				} else if c == '\n' {
					if tree.n_selected == 0 {
						tree.flip_selected(ui.index());
					}
					ui.print_paths(&tree.paths);
					break;
				} else {
					ui.insert_char(c);
				}
			}
			Key::Ctrl(c) => {
				match c {
					'c' => {
						// Make sure we drop ui so that terminal is reverted from "raw mode"
						mem::drop(ui);
						mem::drop(tree);
						process::exit(130);
					}
					'u' => ui.stash(),
					'w' => ui.word_stash(),
					'y' => ui.pop(),
					x => debug!("Got ctrl-{}", x),
				}
			}
			Key::Left => ui.move_left(),
			Key::Right => ui.move_right(),
			Key::Up => ui.move_up(),
			Key::Down => ui.move_down(),
			Key::PageUp => ui.page_up(),
			Key::PageDown => ui.page_down(),
			Key::Backspace => ui.backspace(),
			Key::Delete => ui.delete(),
			Key::Home => ui.home(),
			Key::End => ui.end(),
			x => debug!("Got {:?}", x),
		}

		if ui.chars_changed {
			tree.filter(&ui.current_input());
		}

		let mut info_line = tree.info_line();
		if log_enabled!(Level::Debug) {
			info_line += &ui.info_line();
		}

		ui.render(info_line, tree.as_lines())?;
	}

	ui.flush()?;

	Ok(())
}
