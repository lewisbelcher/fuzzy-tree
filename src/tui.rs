use crate::path;
use std::io::{self, Write};
use termion::cursor::DetectCursorPos;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, color, cursor, scroll};

pub fn println_cleared(s: &str) {
	print!("{}{}\r\n", clear::CurrentLine, s);
}

fn print_tree(paths: &[path::Path], pos: u16, indices: &[usize], display_lines: usize) {
	let highlight = format!(
		"{}{}>",
		color::Bg(color::Rgb(50, 50, 50)),
		color::Fg(color::Red)
	);
	let selected = format!("{}>", color::Fg(color::LightRed));

	for (i, idx) in indices.iter().enumerate() {
		if i == display_lines {
			break;
		}

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

pub fn iter_keys() -> termion::input::Keys<io::Stdin> {
	io::stdin().keys()
}

type RawStdout = termion::raw::RawTerminal<io::Stdout>;

pub struct Tui {
	stdout: RawStdout,
	start_pos: (u16, u16),
	prompt: String,
	display_lines: usize,
	pub curs_pos: u16,
	pub line_pos: u16,
}

impl Tui {
	pub fn new(prompt: String, display_lines: usize) -> Self {
		let mut stdout = io::stdout().into_raw_mode().unwrap();
		let mut start_pos = stdout.cursor_pos().unwrap();

		// Scroll up to allow min screen space at bottom of screen
		let size = termion::terminal_size().unwrap();
		let min_line = size.1 - display_lines as u16;
		if min_line < start_pos.1 {
			print!("{}", scroll::Up(start_pos.1 - min_line));
			start_pos.1 = min_line;
		}

		Tui {
			stdout: stdout,
			start_pos: start_pos,
			curs_pos: 0,
			line_pos: 0,
			prompt,
			display_lines,
		}
	}

	pub fn goto_start(&self) {
		print!("{}", cursor::Goto(self.start_pos.0, self.start_pos.1));
	}

	pub fn print_input_line(&self, string: &str) {
		println_cleared(&format!("{}{}", self.prompt, string));
	}

	pub fn print_body(&self, paths: &[path::Path], indices: &[usize]) {
		print!("{}", clear::AfterCursor);
		print_tree(paths, self.line_pos, indices, self.display_lines - 2);
	}

	pub fn return_cursor(&self) {
		print!("{}", cursor::Goto(self.curs_pos + 3, self.start_pos.1));
	}

	pub fn flush(&mut self) {
		self.stdout.flush().unwrap();
	}
}
