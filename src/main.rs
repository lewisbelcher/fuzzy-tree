#![feature(vec_remove_item)]

use std::io;
use std::process::Command;
use termion::color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
pub mod path;
pub mod tui;

fn main() {
	let stdout = Command::new("fd")
		.output()
		.expect("Failed to execute command `fd`")
		.stdout;

	let mut paths: Vec<path::Path> = String::from_utf8(stdout)
		.unwrap()
		.split('\n')
		.filter(|x| !x.is_empty())
		.map(|x| path::Path::new(x.to_string()))
		.collect();

	paths.sort();
	for pth in &paths {
		println!("{:?}", pth);
	}

	let stdin = io::stdin();
	let prompt = format!("{}> {}", color::Fg(color::Blue), color::Fg(color::Reset));
	let stdout = io::stdout().into_raw_mode().unwrap();
	let mut ui = tui::Tui::new(stdout, prompt);
	let mut chars = Vec::new();

	let _size = termion::terminal_size();

	ui.print_input_line("");
	ui.return_cursor();
	ui.flush();

	for c in stdin.keys() {
		match c.unwrap() {
			Key::Esc => break,
			Key::Char(c) => {
				if c == '\t' {
					let pos = ui.line_pos as usize;
					paths[pos].selected = !paths[pos].selected;
				} else if c == '\n' {
					break;
				} else {
					chars.insert(ui.curs_pos as usize, c);
					ui.curs_pos += 1;
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
				if ui.line_pos < (paths.len() as u16 - 1) {
					ui.line_pos += 1;
				}
			}
			Key::Backspace => {
				if ui.curs_pos > 0 {
					chars.remove((ui.curs_pos - 1) as usize);
					ui.curs_pos -= 1;
				}
			}
			Key::Delete => {
				if (ui.curs_pos as usize) < chars.len() {
					chars.remove(ui.curs_pos as usize);
				}
			}
			Key::Home => ui.curs_pos = 0,
			Key::End => ui.curs_pos = chars.len() as u16,
			_ => {}
		}

		ui.goto_start();
		ui.print_input_line(&chars.iter().collect::<String>());
		tui::print_info_line(
			paths
				.iter()
				.filter(|x| x.selected)
				.collect::<Vec<&path::Path>>()
				.len(),
			paths.len(),
			paths.len(),
		);
		ui.print_body(&paths[..]);
		ui.return_cursor();
		ui.flush();
	}
}
