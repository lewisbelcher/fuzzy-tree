pub mod config;
#[macro_use]
pub mod path;
pub mod tree;
pub mod tui;
use std::env;
use std::process::{self, Command};
use termion::color;
use termion::event::Key;

const DISPLAY_LINES: usize = 10;

fn main() {
	let args = env::args().skip(1).collect::<Vec<_>>();

	let stdout = Command::new("fd")
		.args(&args)
		.output()
		.expect("Failed to execute command `fd`")
		.stdout;

	let mut tree = tree::Tree::from_stdout(stdout);

	let prompt = format!("{}> {}", color::Fg(color::Blue), color::Fg(color::Reset));
	let lines = tree.as_lines();
	let mut ui = tui::Tui::new(prompt, DISPLAY_LINES, lines.len());

	ui.render(tree.info_line(), lines);

	for c in tui::iter_keys() {
		ui.chars_changed = false;

		match c.unwrap() {
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
				// TODO: ctrl-arrow is not supported?
				match c {
					'c' => process::exit(130), // TODO: Fix bad rendering after this
					'u' => ui.stash(),
					'w' => ui.word_stash(),
					'y' => ui.pop(),
					_ => {}
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
			_ => {}
		}

		if ui.chars_changed {
			tree.filter(&ui.current_input());
		}

		let mut info_line = tree.info_line();
		if config::debug() {
			info_line += &ui.info_line();
		}

		ui.render(info_line, tree.as_lines());
	}

	ui.flush();
}
