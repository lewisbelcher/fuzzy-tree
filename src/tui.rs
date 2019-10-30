use crate::config;
use crate::path;
use std::cmp;
use std::io::{self, Write};
use termion::cursor::DetectCursorPos;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, color, cursor, scroll};

pub fn println_cleared(s: &str) {
	print!("{}{}\r\n", clear::CurrentLine, s);
}

fn chars_to_str(chars: &Vec<char>) -> String {
	chars.iter().collect::<String>()
}

fn print_tree(lines: &[String], pos: u16, display_lines: usize) {
	let highlight = format!(
		"{}{}>{}",
		color::Bg(color::Rgb(50, 50, 50)),
		color::Fg(color::Red),
		color::Fg(color::Reset),
	);

	for (i, line) in lines.iter().enumerate() {
		if i == display_lines {
			break;
		}

		print!(
			"{}{}{}{}\r\n",
			clear::CurrentLine,
			if i == (pos as usize) { &highlight } else { " " },
			line,
			color::Bg(color::Reset)
		);
	}
}

pub fn print_info_line(text: String) {
	println_cleared(&format!(
		"{}{}{}",
		color::Fg(color::LightGreen),
		text,
		color::Fg(color::Reset),
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
	offset: usize, // TODO: Keep these in a TuiState?
	chars: Vec<char>,
	pub chars_changed: bool,
	curs_pos: u16,
	line_pos: u16,
	current_lines: Option<usize>,
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
			offset: 0,
			chars: Vec::new(),
			chars_changed: false,
			prompt,
			display_lines,
			current_lines: None,
		}
	}

	fn goto_start(&self) {
		print!("{}", cursor::Goto(self.start_pos.0, self.start_pos.1));
	}

	pub fn info_line(&self) -> String {
		format!(
			"offset: {}, line_pos: {}, index: {}",
			self.offset,
			self.line_pos,
			self.index()
		)
	}

	fn print_input_line(&self) {
		println_cleared(&format!("{}{}", self.prompt, &chars_to_str(&self.chars)));
	}

	fn print_body(&self, lines: Vec<String>) {
		print!("{}", clear::AfterCursor);
		print_tree(&lines[self.offset..], self.line_pos, self.display_lines - 2);
	}

	fn return_cursor(&self) {
		print!("{}", cursor::Goto(self.curs_pos + 3, self.start_pos.1));
	}

	pub fn flush(&mut self) {
		self.stdout.flush().unwrap();
	}

	pub fn move_up(&mut self) {
		let x = self.line_pos as usize;
		if x + self.offset == 0 {
			// Do nothing
		} else if self.line_pos == 0 && self.offset > 0 {
			self.offset -= 1;
		} else {
			self.line_pos -= 1;
		}
	}

	/// Move the current index down one. NB `render` must have previously been
	/// called (this is how we know what the current maximum number of lines is).
	pub fn move_down(&mut self) {
		let current_lines = if let Some(current_lines) = self.current_lines {
			if current_lines == 0 {
				return;
			}
			current_lines
		} else {
			panic!("attempted movement before render");
		};

		let x = self.line_pos as usize;
		if x + self.offset == current_lines - 1 {
			// Do nout
		} else if x == self.display_lines - 3 {
			self.offset += 1;
		} else {
			self.line_pos += 1;
		}
	}

	pub fn move_left(&mut self) {
		if self.curs_pos > 0 {
			self.curs_pos -= 1;
		}
	}

	pub fn move_right(&mut self) {
		if (self.curs_pos as usize) < self.chars.len() {
			self.curs_pos += 1;
		}
	}

	pub fn home(&mut self) {
		self.curs_pos = 0;
	}

	pub fn end(&mut self) {
		self.curs_pos = self.chars.len() as u16;
	}

	pub fn insert_char(&mut self, c: char) {
		self.chars.insert(self.curs_pos as usize, c);
		self.curs_pos += 1;
		self.chars_changed = true;
	}

	pub fn backspace(&mut self) {
		if self.curs_pos > 0 {
			self.chars.remove((self.curs_pos - 1) as usize);
			self.curs_pos -= 1;
			self.chars_changed = true;
		}
	}

	pub fn delete(&mut self) {
		if (self.curs_pos as usize) < self.chars.len() {
			self.chars.remove(self.curs_pos as usize);
			self.chars_changed = true;
		}
	}

	pub fn print_paths(&mut self, paths: &Vec<path::RcPath>) {
		self.goto_start();
		print!("{}", clear::AfterCursor);
		let _ = paths
			.iter()
			.map(|p| {
				let p = p.borrow();
				if p.selected {
					print!("{} ", &p.joined);
				}
			})
			.collect::<()>();
	}

	fn adjust_offset(&mut self, new_len: usize) {
		let current_lines = self.current_lines.unwrap();
		if new_len < current_lines {
			let diff = cmp::min(self.offset, current_lines - new_len);
			self.offset -= diff;
		}
	}

	pub fn render(&mut self, info_line: String, path_lines: Vec<String>) {
		if self.chars_changed {
			self.adjust_offset(path_lines.len());
			let x = cmp::max(1, path_lines.len()) - 1;
			self.line_pos = cmp::min(self.line_pos, x as u16);
			self.offset = cmp::min(self.offset, x)
		}

		self.current_lines = Some(path_lines.len());
		if !config::debug() {
			self.goto_start();
		}
		self.print_input_line();
		print_info_line(info_line);
		self.print_body(path_lines);
		if !config::debug() {
			self.return_cursor();
		}
		self.flush();
	}

	/// Return the total index position, defined as the current line number
	/// plus the offset into the displayed lines.
	pub fn index(&self) -> usize {
		self.line_pos as usize + self.offset
	}

	/// Return the current command line input as a string.
	pub fn current_input(&self) -> String {
		chars_to_str(&self.chars)
	}
}
