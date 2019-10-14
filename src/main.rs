#[macro_use]
extern crate lazy_static;
pub mod path;
pub mod tui;
use std::process::Command;
use termion::color;
use termion::event::Key;

const DISPLAY_LINES: usize = 10;

fn main() {
	let stdout = Command::new("fd")
		.output()
		.expect("Failed to execute command `fd`")
		.stdout;

	let mut tree = path::Tree::from_stdout(stdout);

	let prompt = format!("{}> {}", color::Fg(color::Blue), color::Fg(color::Reset));
	let mut ui = tui::Tui::new(prompt, DISPLAY_LINES);

	ui.render(tree.info_line(), tree.as_lines());

	for c in tui::iter_keys() {
		ui.chars_changed = false;

		match c.unwrap() {
			Key::Esc => break,
			Key::Char(c) => {
				if c == '\t' {
					tree.flip_selected(ui.index());
					ui.move_down();
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
			Key::Ctrl(_c) => {}
			Key::Left => ui.move_left(),
			Key::Right => ui.move_right(),
			Key::Up => ui.move_up(),
			Key::Down => ui.move_down(),
			Key::Backspace => ui.backspace(),
			Key::Delete => ui.delete(),
			Key::Home => ui.home(),
			Key::End => ui.end(),
			_ => {}
		}

		if ui.chars_changed {
			tree.filter(&ui.current_input());
		}
		ui.render(tree.info_line(), tree.as_lines());
	}

	ui.flush();
}
