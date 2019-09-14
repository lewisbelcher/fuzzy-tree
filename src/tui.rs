extern crate termion;

use crate::path;
use std::io::{self, Write};
use termion::cursor::DetectCursorPos;
use termion::{clear, color, cursor};

pub fn println_cleared(s: &str) {
	print!("{}{}\r\n", clear::CurrentLine, s);
}

pub fn clear_lines(n: usize) {
	for _ in 0..n {
		print!("{}\r\n", clear::CurrentLine);
	}
}

fn print_tree(paths: &[path::Path], pos: u16, indices: &[usize]) {
	let highlight = format!(
		"{}{}>",
		color::Bg(color::Rgb(50, 50, 50)),
		color::Fg(color::Red)
	);
	let selected = format!("{}>", color::Fg(color::LightRed));

	for (i, idx) in indices.iter().enumerate() {
		let pth = &paths[*idx];
		print!(
			"{}{}{}{} {:?}{}\r\n",
			clear::CurrentLine,
			if i == (pos as usize) { &highlight } else { " " },
			if pth.selected { &selected } else { " " },
			color::Fg(color::Reset),
			pth,
			color::Bg(color::Reset)
		);
	}
}

pub fn print_info_line(n_selected: usize, n_shown: usize, n_total: usize) {
	println_cleared(&format!(
		"{}(selected: {}, shown: {}, total: {})",
		color::Fg(color::LightGreen),
		n_selected,
		n_shown,
		n_total,
	));
}

type RawStdout = termion::raw::RawTerminal<io::Stdout>;

pub struct Tui {
	pub start_pos: (u16, u16),
	pub curs_pos: u16,
	pub line_pos: u16,
	pub stdout: RawStdout,
	pub prompt: String,
	max_lines: usize,
}

impl Tui {
	pub fn new(mut stdout: RawStdout, prompt: String, max_lines: usize) -> Self {
		Tui {
			start_pos: stdout.cursor_pos().unwrap(),
			curs_pos: 0,
			line_pos: 0,
			stdout: stdout,
			prompt,
			max_lines,
		}
	}

	pub fn goto_start(&self) {
		print!("{}", cursor::Goto(self.start_pos.0, self.start_pos.1));
	}

	pub fn print_input_line(&self, string: &str) {
		println_cleared(&format!("{}{}", self.prompt, string));
	}

	pub fn print_body(&self, paths: &[path::Path], indices: &[usize]) {
		print_tree(paths, self.line_pos, indices);
		clear_lines(self.max_lines - indices.len());
	}

	pub fn return_cursor(&self) {
		print!("{}", cursor::Goto(self.curs_pos + 3, self.start_pos.1));
	}

	pub fn flush(&mut self) {
		self.stdout.flush().unwrap();
	}
}
