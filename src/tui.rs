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
			"{}{}{}{}{}",
			clear::CurrentLine,
			if i == (pos as usize) { &highlight } else { " " },
			line,
			color::Bg(color::Reset),
			if i == display_lines - 1 { "" } else { "\r\n" }, // TODO: Optimise
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
	stash: Vec<char>,
	pub chars_changed: bool,
	curs_pos: u16,
	line_pos: u16,
	current_lines: usize,
}

impl Tui {
	pub fn new(prompt: String, mut display_lines: usize, current_lines: usize) -> Self {
		let mut stdout = io::stdout().into_raw_mode().unwrap();
		let mut start_pos = stdout.cursor_pos().unwrap();

		// Scroll up to allow min screen space at bottom of screen
		let size = termion::terminal_size().unwrap();
		display_lines = cmp::min(display_lines, size.1 as usize);
		debug!("Terminal size: {:?}", size);
		debug!("Starting pos: {:?}", start_pos);
		let min_line = size.1 - display_lines as u16;
		if min_line < start_pos.1 {
			let diff = start_pos.1 - min_line;
			debug!("Scrolling up {} lines", diff);
			print!("{}", scroll::Up(diff));
			start_pos.1 = min_line;
		}

		Tui {
			stdout: stdout,
			start_pos: start_pos,
			curs_pos: 0,
			line_pos: 0,
			offset: 0,
			chars: Vec::new(),
			stash: Vec::new(),
			chars_changed: false,
			prompt,
			display_lines,
			current_lines,
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
		if self.current_lines == 0 {
			return;
		}

		let x = self.line_pos as usize;
		if x + self.offset == self.current_lines - 1 {
			// Do nout
		} else if x == self.display_lines - 3 {
			self.offset += 1;
		} else {
			self.line_pos += 1;
		}
	}

	pub fn page_up(&mut self) {
		if self.offset >= self.display_lines {
			self.offset -= self.display_lines;
		} else {
			if self.offset == 0 {
				self.line_pos = 0;
			} else {
				self.offset = 0;
			}
		}
	}

	pub fn page_down(&mut self) {
		if self.current_lines == 0 {
		} else if self.current_lines <= self.offset + self.display_lines {
			self.offset = self.current_lines - 1;
			self.line_pos = 0;
		} else {
			self.offset += self.display_lines;

			if self.index() >= self.current_lines {
				let diff = (self.index() - self.current_lines + 1) as u16;
				self.line_pos -= cmp::min(self.line_pos, diff);
			}
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

	pub fn move_left_word(&mut self) {
		let curs_pos = self.curs_pos as usize;
		if self.curs_pos > 0 {
			for i in 0..curs_pos {
				if self.chars[curs_pos - i] == ' ' {
					self.curs_pos = i as u16;
					break;
				}
			}
		}
	}

	pub fn word_stash(&mut self) {
		if self.curs_pos < 1 {
			return;
		}

		let mut curs_pos = (self.curs_pos as usize) - 1;
		let mut popped = Vec::new();
		let mut seen_char = false;

		// TODO: Clean this up
		loop {
			if self.chars[curs_pos] == ' ' && seen_char {
				curs_pos += 1;
				break;
			}
			let c = self.chars.remove(curs_pos);
			if c != ' ' {
				seen_char = true;
			}

			popped.push(c);
			curs_pos -= 1;
			if curs_pos == 0 {
				popped.push(self.chars.remove(curs_pos));
				break;
			}
		}
		self.chars_changed = true;
		popped.reverse();
		self.stash = popped;
		self.curs_pos = curs_pos as u16;
	}

	pub fn stash(&mut self) {
		let (stash, chars) = self.chars.split_at(self.curs_pos as usize);
		self.stash = stash.to_vec();
		self.chars = chars.to_vec();
		self.curs_pos = 0;
		self.chars_changed = true;
	}

	pub fn pop(&mut self) {
		let curs_pos = self.curs_pos as usize;
		self.curs_pos += self.stash.len() as u16;
		self.chars = [
			&self.chars[..curs_pos],
			&self.stash[..],
			&self.chars[curs_pos..],
		]
		.concat();
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
		if new_len < self.current_lines {
			let diff = cmp::min(self.offset, self.current_lines - new_len);
			self.offset -= diff;
		}
	}

	pub fn render(&mut self, info_line: String, path_lines: Vec<String>) {
		if self.chars_changed && self.index() >= path_lines.len() {
			self.adjust_offset(path_lines.len());
			let x = cmp::max(1, path_lines.len()) - 1;
			self.line_pos = cmp::min(self.line_pos, x as u16);
		}

		self.current_lines = path_lines.len();
		self.goto_start();
		self.print_input_line();
		print_info_line(info_line);
		self.print_body(path_lines);
		self.return_cursor();
		self.flush();
		self.chars_changed = false;
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
