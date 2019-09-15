#![feature(vec_remove_item)]

use std::cmp;
use std::process::Command;
use termion::clear;
use termion::color;
use termion::event::Key;
pub mod path;
pub mod tui;

const DISPLAY_LINES: usize = 10;

fn chars_to_str(chars: &Vec<char>) -> String {
	chars.iter().collect::<String>()
}

fn main() {
	let stdout = Command::new("fd")
		.output()
		.expect("Failed to execute command `fd`")
		.stdout;

	let mut paths = path::create_paths(stdout);
	let n_paths = paths.len();

	let prompt = format!("{}> {}", color::Fg(color::Blue), color::Fg(color::Reset));
	let mut ui = tui::Tui::new(prompt, DISPLAY_LINES);
	let mut chars = Vec::new();
	let mut indices: Vec<usize> = (0..paths.len()).collect();
	let mut n_selected: usize = 0;

	ui.goto_start();
	ui.print_input_line("");
	ui.return_cursor();
	ui.flush();

	for c in tui::iter_keys() {
		let mut chars_changed = false;

		match c.unwrap() {
			Key::Esc => break,
			Key::Char(c) => {
				if c == '\t' {
					let idx = indices[ui.line_pos as usize];
					if paths[idx].selected {
						paths[idx].selected = false;
						n_selected -= 1;
					} else {
						paths[idx].selected = true;
						n_selected += 1;
					}
				} else if c == '\n' {
					ui.goto_start();
					print!("{}", clear::CurrentLine);
					let _ = paths
						.iter()
						.map(|p| {
							if p.selected {
								p.print_joined()
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
			Key::Ctrl(_c) => {} // TODO: implement ctrl+{w,u,y,left,right}
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
				if ui.line_pos > 0 {
					ui.line_pos -= 1;
				}
			}
			Key::Down => {
				let min = cmp::min(indices.len(), DISPLAY_LINES - 2);
				if min > 0 && ui.line_pos < (min as u16 - 1) {
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
			indices = path::filter(&paths[..], &chars_to_str(&chars));
			ui.line_pos = cmp::min(ui.line_pos, (cmp::max(1, indices.len()) - 1) as u16);
		}

		ui.goto_start();
		ui.print_input_line(&chars_to_str(&chars));
		tui::print_info_line(n_selected, indices.len(), n_paths);
		ui.print_body(&paths[..], &indices[..]);
		ui.return_cursor();
		ui.flush();
	}
	ui.flush();
}
