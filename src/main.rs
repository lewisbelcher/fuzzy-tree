#![feature(vec_remove_item)]

#[macro_use]
extern crate lazy_static;
pub mod path;
pub mod tui;
use path::PathBehaviour;
use std::cmp;
use std::process::Command;
use termion::clear;
use termion::color;
use termion::event::Key;

const DISPLAY_LINES: usize = 10;

fn chars_to_str(chars: &Vec<char>) -> String {
	chars.iter().collect::<String>()
}

fn main() {
	let stdout = Command::new("fd")
		.output()
		.expect("Failed to execute command `fd`")
		.stdout;

	let paths = path::create_paths(stdout);
	let n_paths = paths.len();
	let root = path::create_tree(&paths);

	let prompt = format!("{}> {}", color::Fg(color::Blue), color::Fg(color::Reset));
	let mut ui = tui::Tui::new(prompt, DISPLAY_LINES);
	let mut chars = Vec::new();
	let mut n_matches = n_paths;
	let mut n_selected: usize = 0;
	let mut offset = 0;

	ui.goto_start();
	ui.print_input_line("");
	tui::print_info_line(n_selected, n_matches, n_paths);
	ui.print_body(&path::tree_string(&root, n_paths)[offset..]);
	ui.return_cursor();
	ui.flush();

	let mut chars_changed = false;

	for c in tui::iter_keys() {
		match c.unwrap() {
			Key::Esc => break,
			Key::Char(c) => {
				if c == '\t' {
					let mut pth = path::get_n(&paths, ui.line_pos as usize + offset)
						.unwrap()
						.borrow_mut();
					if pth.selected {
						pth.selected = false;
						n_selected -= 1;
					} else {
						pth.selected = true;
						n_selected += 1;
					}
				} else if c == '\n' {
					ui.goto_start();
					print!("{}", clear::AfterCursor);
					let _ = paths
						.iter()
						.map(|p| {
							if p.borrow().selected {
								print!("{} ", p.joined());
							}
						})
						.collect::<()>();
					break;
				} else {
					chars.insert(ui.curs_pos as usize, c);
					ui.curs_pos += 1;
					chars_changed = true;
				}
			}
			Key::Ctrl(_c) => {}
			Key::Left => {
				if ui.curs_pos > 0 {
					ui.curs_pos -= 1;
				}
			}
			Key::Right => {
				if (ui.curs_pos as usize) < chars.len() {
					ui.curs_pos += 1;
				}
			}
			Key::Up => {
				let x = ui.line_pos as usize;
				if x + offset == 0 {
					// Do nout
				} else if ui.line_pos == 0 && offset > 0 {
					offset -= 1;
				} else {
					ui.line_pos -= 1;
				}
			}
			Key::Down => {
				let x = ui.line_pos as usize;
				if x + offset == n_matches - 1 {
					// Do nout
				} else if x == DISPLAY_LINES - 3 {
					offset += 1;
				} else {
					ui.line_pos += 1;
				}
			}
			Key::Backspace => {
				if ui.curs_pos > 0 {
					chars.remove((ui.curs_pos - 1) as usize);
					ui.curs_pos -= 1;
					chars_changed = true;
				}
			}
			Key::Delete => {
				if (ui.curs_pos as usize) < chars.len() {
					chars.remove(ui.curs_pos as usize);
					chars_changed = true;
				}
			}
			Key::Home => ui.curs_pos = 0,
			Key::End => ui.curs_pos = chars.len() as u16,
			_ => {}
		}

		if chars_changed {
			n_matches = path::update_matched(&paths, &chars_to_str(&chars));
			let x = cmp::max(1, n_matches) - 1;
			ui.line_pos = cmp::min(ui.line_pos, x as u16);
			offset = cmp::min(offset, x)
		}

		ui.goto_start();
		ui.print_input_line(&chars_to_str(&chars));
		tui::print_info_line(n_selected, n_matches, n_paths);
		ui.print_body(&path::tree_string(&root, n_paths)[offset..]);
		ui.return_cursor();
		ui.flush();
	}
	ui.flush();
}
