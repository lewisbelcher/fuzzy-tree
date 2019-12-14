use log::debug;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;
use std::fs;
use std::path;
use std::rc::Rc;
use termion::color;

pub type RcPath = Rc<RefCell<Path>>;

const DIR_OPEN: &str = "  ";
const DIR_CLOSED: &str = "  ";
const BLUE: &str = "\u{1b}[38;5;12m";
const RESET: &str = "\u{1b}[39m";
const COLOR_WRAP_LEN: usize = 15;

#[derive(Eq, PartialEq)]
pub struct Path {
	components: Vec<String>,
	parent: Option<RcPath>,
	children: Option<Vec<RcPath>>,
	is_dir: bool,
	open: bool,
	matched: bool,
	match_text: String,
	pub selected: bool,
	pub joined: String,
}

impl fmt::Debug for Path {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{:?}; Selected: {}; Matched {}; Children {:#?}:",
			self.components, self.selected, self.matched, self.children
		)
	}
}

impl Ord for Path {
	fn cmp(&self, other: &Self) -> Ordering {
		let mut result;

		for (x, y) in self.components.iter().zip(other.components.iter()) {
			result = x.cmp(y);
			if result != Ordering::Equal {
				return result;
			}
		}

		// If all zipped components were equal, compare length
		self.components.len().cmp(&other.components.len())
	}
}

impl PartialOrd for Path {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Path {
	pub fn new(pathname: String, is_dir: bool) -> RcPath {
		let components: Vec<String> = pathname
			.split(path::MAIN_SEPARATOR)
			.map(|x| x.to_string())
			.collect();
		let match_text = components[components.len() - 1].clone();

		Rc::new(RefCell::new(Path {
			parent: None,
			components,
			joined: pathname,
			selected: false,
			matched: true,
			match_text,
			is_dir,
			open: true,
			children: None,
		}))
	}

	pub fn from(pathname: &str, is_dir: bool) -> RcPath {
		Path::new(pathname.to_string(), is_dir)
	}
}

fn add(child: &RcPath, parent: &RcPath) {
	let mut children = match parent.borrow_mut().children.take() {
		Some(v) => v,
		None => Vec::new(),
	};
	children.push(Rc::clone(child));
	parent.borrow_mut().children = Some(children);
	child.borrow_mut().parent = Some(Rc::clone(parent));
}

// Since `Path`s are wrapped in `Rc`s which cannot be implemented on directly,
// functionality is implemented via a trait.
pub trait PathBehaviour {
	fn add_child(&self, child: &RcPath);
	fn add_parent(&self, parent: &RcPath);
	fn is_child_of(&self, other: &RcPath) -> bool;
	fn basename(&self) -> &str;
	fn len(&self) -> usize;
	fn n_descendants(&self) -> usize;
}

impl PathBehaviour for RcPath {
	// TODO: Also add helper methods for borrow of `matched`, `selected`, `children`
	// etc.. This could be done with a macro?

	fn add_child(&self, child: &RcPath) {
		add(child, self);
	}

	fn add_parent(&self, parent: &RcPath) {
		add(self, parent);
	}

	fn is_child_of(&self, other: &RcPath) -> bool {
		if other.len() >= self.len() {
			return false;
		}
		self.borrow().components[..other.len()] == other.borrow().components[..]
	}

	fn basename(&self) -> &str {
		// TODO: See if there is a safe way around this:
		unsafe { &(*self.as_ptr()).components[self.len() - 1] }
	}

	fn len(&self) -> usize {
		self.borrow().components.len()
	}

	fn n_descendants(&self) -> usize {
		let mut i = 0;
		if let Some(children) = &self.borrow().children {
			for child in children.iter() {
				i += child.n_descendants() + 1;
			}
		}
		i
	}
}

pub struct Tree {
	pub paths: Vec<RcPath>,
	pub tree: RcPath,
	pub n_paths: usize,
	pub n_matches: usize,
	pub n_selected: usize,
}

impl Tree {
	pub fn from_stdout(stdout: Vec<u8>) -> Self {
		let paths = create_paths(stdout);
		Self::from_paths(paths)
	}

	pub fn from_paths(paths: Vec<RcPath>) -> Self {
		let tree = create_tree(&paths);
		let n_paths = paths.len();

		Self {
			paths,
			tree,
			n_paths,
			n_matches: n_paths,
			n_selected: 0,
		}
	}

	fn reset_matched(&self, value: bool) {
		for path in &self.paths {
			let basename = path.basename().to_string();
			let mut pth = path.borrow_mut();
			pth.matched = value;
			pth.match_text = basename;
		}
	}

	pub fn filter(&mut self, text: &str) {
		if text.is_empty() {
			self.reset_matched(true);
			self.n_matches = self.paths.len();
		} else {
			self.reset_matched(false);
			let patterns: Vec<&str> = text.split(" ").filter(|x| !x.is_empty()).collect();
			let patterns = reduce_patterns(&patterns);
			matchfn(&self.paths, &patterns);
			self.n_matches = self.calc_n_matches();
		}
	}

	fn calc_n_matches(&self) -> usize {
		self.paths
			.iter()
			.filter(|p| p.borrow().matched)
			.collect::<Vec<_>>()
			.len()
	}

	pub fn as_lines(&self) -> Vec<String> {
		tree_string(&self.tree, self.n_matches)
	}

	pub fn info_line(&self) -> String {
		format!(
			"(selected: {}, shown: {}, total: {})",
			self.n_selected, self.n_matches, self.n_paths,
		)
	}

	/// Get the i'th visible path. Returns `None` if `target` is out of range.
	fn ith(&self, mut target: usize) -> Option<&RcPath> {
		let mut i = 0;
		loop {
			let pth = self.paths.get(i)?;
			if !pth.borrow().matched {
				target += 1;
			}
			if i == target {
				return Some(pth);
			}
			if !pth.borrow().open {
				let n_descendants = pth.n_descendants();
				target += n_descendants;
				i += n_descendants;
			}
			i += 1;
		}
	}

	/// Flip the `open` status of the `i`th displayed path.
	pub fn flip_open(&mut self, i: usize) {
		let mut pth;
		if let Some(_pth) = self.ith(i) {
			pth = _pth.borrow_mut();
			pth.open = !pth.open;
		}
	}

	/// Flip the `selected` status of the `i`th displayed path.
	pub fn flip_selected(&mut self, i: usize) {
		{
			let mut pth;
			if let Some(_pth) = self.ith(i) {
				pth = _pth.borrow_mut();
				pth.selected = !pth.selected;
			} else {
				return;
			}
		}

		let mut n_selected = 0;
		self.paths
			.iter()
			.map(|p| {
				if p.borrow().selected {
					n_selected += 1
				}
			})
			.for_each(drop);
		self.n_selected = n_selected;
	}
}

// TODO: Should be able to use node directly instead of a clone of the
// joined path....
fn push_seen(seen: &mut Vec<String>, node: &RcPath) -> bool {
	let rf = &node.borrow().joined;
	if seen.contains(rf) {
		false
	} else {
		seen.push(rf.clone());
		true
	}
}

// NB Since paths are assumed to be sorted, we assume that we'll iterate
// children after parents
fn match_stack(node: &RcPath, seen: &mut Vec<String>) -> usize {
	let mut n = 1;
	node.borrow_mut().matched = true;
	push_seen(seen, &node);

	if let Some(parent) = &node.borrow().parent {
		if push_seen(seen, &parent) {
			n += match_stack(parent, seen);
		}
	}
	return n;
}

/// Reduce a vector of patterns to contain only elements which are disjoint
fn reduce_patterns<'a>(patterns: &Vec<&'a str>) -> Vec<&'a str> {
	let mut rm = Vec::new();

	for (i, pat1) in patterns.iter().enumerate() {
		for pat2 in patterns {
			if pat1 == pat2 {
				// skip
			} else if pat2.contains(pat1) {
				rm.push(i);
			}
		}
	}

	let mut patterns = patterns
		.iter()
		.enumerate()
		.filter_map(|(i, x)| if rm.contains(&i) { None } else { Some(*x) })
		.collect::<Vec<&str>>();
	patterns.sort();
	patterns.dedup();
	patterns
}

/// Works under the assumption that all patterns are disjoint patterns. Use
/// `reduce_patterns` to ensure this.
fn matchfn(paths: &Vec<RcPath>, patterns: &Vec<&str>) {
	let mut seen = Vec::new();

	for path in paths {
		let matched;
		{
			let joined = &path.borrow().joined;
			// Total match:
			matched = patterns.iter().all(|pat| joined.contains(pat));
			// Partial match:
			// matched = patterns.iter().any(|pat| joined.contains(pat));
		}

		if matched {
			// TODO: Abstract the match function to implement a trait and use this
			// in reduce_patterns too.
			let basename = &path.basename();

			let mut match_idxs: Vec<(usize, usize)> = patterns
				.iter()
				.flat_map(|p| basename.match_indices(p).map(move |(idx, _)| (idx, p.len())))
				.collect();

			let mut match_text: String;
			if match_idxs.is_empty() {
				match_text = basename.to_string()
			} else {
				match_idxs.sort();
				match_text =
					String::with_capacity(basename.len() + COLOR_WRAP_LEN * match_idxs.len());
				let mut _mm = match_idxs.into_iter();
				let (mut i, mut len) = _mm.next().unwrap();

				for (j, c) in basename.chars().enumerate() {
					if j == i {
						match_text.push_str(BLUE);
					} else if j == i + len {
						match_text.push_str(RESET);
						if let Some((ii, llen)) = _mm.next() {
							i = ii;
							len = llen;
						} else {
							match_text.push_str(&basename[j..]);
							break;
						}
					}
					match_text.push(c);
				}
				if i + len == basename.len() {
					match_text.push_str(RESET);
				}
			}

			match_stack(path, &mut seen);
			path.borrow_mut().match_text = match_text;
		}
	}
}

/// Create multiple paths from a `find`-like command output.
pub fn create_paths(string: Vec<u8>) -> Vec<RcPath> {
	let mut paths: Vec<RcPath> = String::from_utf8(string)
		.unwrap()
		.split('\n')
		.filter(|x| !x.is_empty())
		.map(|x| Path::from(x, fs::metadata(&x).unwrap().is_dir()))
		.collect();

	paths.sort();

	// Add CWD as "."
	for p in &mut paths {
		p.borrow_mut().components.insert(0, ".".to_string());
	}
	paths.insert(0, Path::from(".", true));

	paths
}

fn peek(stack: &[RcPath], i: usize) -> Option<&RcPath> {
	if i < stack.len() {
		return Some(&stack[i]);
	}
	None
}

macro_rules! debug_relation {
	($obj1:expr; child $obj2:expr) => {
		debug!("{:?} is child of {:?}", $obj1.borrow(), $obj2.borrow());
	};
	($obj1:expr; unrelated $obj2:expr, $obj3:expr) => {
		debug!(
			"{:?} is unrelated to {:?} and {:?}",
			$obj1.borrow(),
			$obj2.borrow(),
			$obj3.borrow()
		);
	};
}

/// Inner recursive function for creating a directory tree.
///
/// Max recursion depth will be equal to the max directory depth. It is
/// unlikely that we'll hit a stack overflow.
///
/// There are three potential routes in each recursion frame:
///  1. If `next` is child of `prev`: add `next` to `prev` and recurse with
///     `prev` as `base`, `next` as `prev`
///  2. If `next` is a child of `base`: consume `next` and add it to `base`
///     then loop
///  3. In all other cases: break with `i` and move up one recursion frame
fn _create_tree<'a>(
	mut i: usize,
	base: &RcPath,
	mut prev: Option<&'a RcPath>,
	stack: &'a [RcPath],
) -> usize {
	// TODO: Use log package
	loop {
		debug!("~~~ {} ~~~", i);
		i += 1;
		if let Some(next) = peek(stack, i) {
			if let Some(prev) = prev {
				if next.is_child_of(&prev) {
					debug_relation!(next; child base);
					prev.add_child(&next);
					i = _create_tree(i, &prev, Some(&next), stack) - 1;
				} else if next.is_child_of(&base) {
					debug_relation!(next; child base);
					base.add_child(&next);
				} else {
					debug_relation!(next; unrelated base, prev);
					break i;
				}
			} else if next.is_child_of(&base) {
				debug_relation!(next; child base);
				base.add_child(&next);
			} else {
				// Given that we always include all directories, there is never a broken
				// link between `base` and `next` if `prev` is `None`. This means that
				// this code is unreachable. It's left here in case the directory
				// discovery changes in the future.
				break i;
			}
			prev = Some(next);
		} else {
			break i;
		}
	}
}

/// Create relationships between all nodes in the directory structure for
/// `paths`.
pub fn create_tree(paths: &Vec<RcPath>) -> RcPath {
	_create_tree(0, &paths[0], None, &paths[..]);
	Rc::clone(&paths[0])
}

#[derive(Clone)]
enum Segment {
	Continuation, // "│   " up to basename, "├── " at basename
	End,          // "    " up to basename, "└── " at basename
}

fn segments_to_string(segments: &Vec<Segment>) -> String {
	// Each char is 4 bytes, each string representation is 4 chars
	let mut s = String::with_capacity(4 * 4 * segments.len());

	if segments.is_empty() {
		return s;
	}

	for seg in segments[..segments.len() - 1].iter() {
		s.push_str(match seg {
			Segment::Continuation => "│   ",
			Segment::End => "    ",
		});
	}
	s.push_str(match segments[segments.len() - 1] {
		Segment::Continuation => "├── ",
		Segment::End => "└── ",
	});
	s
}

/// Inner recursive function to create a string representation of a directory
/// tree.
fn _tree_string(node: &RcPath, lines: &mut Vec<String>, segments: Vec<Segment>) {
	let sel = if node.borrow().selected {
		lazy_static! {
			static ref SELECTED: String =
				format!("{}>{}", color::Fg(color::LightRed), color::Fg(color::Reset));
		}
		&SELECTED
	} else {
		" "
	};

	let prefix = if node.borrow().is_dir {
		if node.borrow().open {
			DIR_OPEN
		} else {
			DIR_CLOSED
		}
	} else {
		""
	};

	// lines.push(sel.to_owned() + &segments_to_string(&segments) + prefix + node.basename());
	lines
		.push(sel.to_owned() + &segments_to_string(&segments) + prefix + &node.borrow().match_text);

	if node.borrow().open {
		if let Some(children) = &node.borrow().children {
			let children: Vec<&RcPath> = children.iter().filter(|x| x.borrow().matched).collect();
			for (i, child) in children.iter().enumerate() {
				let mut segments = segments.clone();

				segments.push(if i == children.len() - 1 {
					Segment::End
				} else {
					Segment::Continuation
				});
				_tree_string(child, lines, segments);
			}
		}
	}
}

/// Create a vec of strings representing the directory tree `tree`. We can
/// preallocate the exact capacity by knowing the number of paths we are
/// constructing for.
pub fn tree_string(tree: &RcPath, len: usize) -> Vec<String> {
	let mut lines = Vec::with_capacity(len);
	if len > 0 {
		_tree_string(tree, &mut lines, Vec::new());
	}
	lines
}

#[cfg(test)]
mod test {
	use super::*;

	macro_rules! paths {
		( $( $x:literal ),* ) => {
			{
				let mut temp = Vec::new();
				$(
					let is_dir = $x.matches('.').collect::<Vec<_>>().len() == 1;
					temp.push(Path::from($x, is_dir));
				)*
				temp
			}
		};
	}

	#[test]
	fn sorting_is_correct() {
		let mut path1;
		let mut path2;

		path1 = Path::from("here/is/a/path.c", false);
		path2 = Path::from("here/is/a/path.c", false);
		assert_eq!(path1, path2);

		path1 = Path::from("here/is/a", false);
		path2 = Path::from("here/is/a/path.c", false);
		assert!(path1 < path2);

		path1 = Path::from("here/is/a/fath.c", false);
		path2 = Path::from("here/is/a/path.c", false);
		assert!(path1 < path2);

		let mut paths = paths!["src", "tmp", "src/main.rs"];
		paths.sort();
		let expected = paths!["src", "src/main.rs", "tmp"];
		assert_eq!(paths, expected);
	}

	#[test]
	fn len_correct() {
		let s = "here/is/a/path.c";
		let path = Path::from(s, false);
		assert_eq!(path.len(), 4);
	}

	#[test]
	fn basename_correct() {
		let s = "here/is/a/path.c";
		let path = Path::from(s, false);
		assert_eq!(path.basename(), "path.c");
	}

	#[test]
	fn is_child_of_correctness() {
		let mut p1 = Path::from("A", false);
		let mut p2 = Path::from("B", false);
		assert!(!p1.is_child_of(&p2));

		p1 = Path::from("src/bayes", true);
		p2 = Path::from("src/bayes/blend.c", false);
		assert!(!p1.is_child_of(&p2));

		p1 = Path::from("src/bayes", true);
		p2 = Path::from("src/bayes/blend.c", false);
		assert!(p2.is_child_of(&p1));
	}

	/// Determine whether two trees are equal by recursing into all branches
	fn trees_equal(a: &RcPath, b: &RcPath) -> bool {
		if a != b {
			return false;
		}

		if let Some(a_children) = &a.borrow().children {
			if let Some(b_children) = &b.borrow().children {
				if a_children.len() != b_children.len() {
					return false;
				}
				for (aa, bb) in a_children.iter().zip(b_children.iter()) {
					if !trees_equal(aa, bb) {
						return false;
					}
				}
			} else {
				return false;
			}
		}

		true
	}

	fn create_test_paths() -> Vec<RcPath> {
		paths![
			".",
			"./A",
			"./B",
			"./src",
			"./src/bayes",
			"./src/bayes/blend.c",
			"./src/bayes/rand.c",
			"./src/cakes",
			"./src/cakes/a.c",
			"./src/cakes/b.c",
			"./x.txt"
		]
	}

	fn create_test_tree(paths: &Vec<RcPath>) -> RcPath {
		let root = Rc::clone(&paths[0]);
		root.add_child(&paths[1]);
		root.add_child(&paths[2]);
		root.add_child(&paths[3]);
		&paths[3].add_child(&paths[4]);
		&paths[4].add_child(&paths[5]);
		&paths[4].add_child(&paths[6]);
		&paths[3].add_child(&paths[7]);
		&paths[7].add_child(&paths[8]);
		&paths[7].add_child(&paths[9]);
		root.add_child(&paths[10]);
		root
	}

	#[test]
	fn create_tree_correct() {
		let paths = create_test_paths();
		let tree = create_tree(&paths);
		let tree2 = create_test_tree(&paths);
		assert!(trees_equal(&tree, &tree2));
	}

	#[test]
	fn tree_string_correct() {
		let mut paths = create_test_paths();
		let tree = create_test_tree(&paths);
		let lines = tree_string(&tree, paths.len());
		let expected = vec![
			"   .",
			" ├──   A",
			" ├──   B",
			" ├──   src",
			" │   ├──   bayes",
			" │   │   ├── blend.c",
			" │   │   └── rand.c",
			" │   └──   cakes",
			" │       ├── a.c",
			" │       └── b.c",
			" └── x.txt",
		];
		assert_eq!(lines, expected);

		// Deselect `./src/bayes` and print again
		paths[4].borrow_mut().matched = false;
		let lines = tree_string(&tree, paths.len());
		let expected = vec![
			"   .",
			" ├──   A",
			" ├──   B",
			" ├──   src",
			" │   └──   cakes",
			" │       ├── a.c",
			" │       └── b.c",
			" └── x.txt",
		];
		assert_eq!(lines, expected);
	}

	#[test]
	fn correct_lines_after_filtering() {
		let paths = create_test_paths();
		let mut tree = Tree::from_paths(paths);
		tree.filter("b");
		let lines = tree.as_lines();
		let colored = vec![
			format!("     ├──   {}b{}ayes", BLUE, RESET),
			format!("     │   ├── {}b{}lend.c", BLUE, RESET),
			format!("         └── {}b{}.c", BLUE, RESET),
		];
		let expected = vec![
			"   .",
			" └──   src",
			&colored[0],
			&colored[1],
			"     │   └── rand.c",
			"     └──   cakes",
			&colored[2],
		];
		assert_eq!(tree.calc_n_matches(), expected.len());
		assert_eq!(lines, expected);
	}

	#[test]
	fn correct_n_matched_after_matching_with_empty_string() {
		let paths = create_test_paths();
		let mut tree = Tree::from_paths(paths);
		tree.filter("");
		assert_eq!(tree.calc_n_matches(), tree.paths.len());
	}

	#[test]
	fn update_matched_test() {
		let paths = create_test_paths();
		let mut tree = Tree::from_paths(paths);

		tree.filter("tmp");
		assert_eq!(tree.calc_n_matches(), 0);

		tree.filter("src");
		assert_eq!(tree.calc_n_matches(), 8);

		let mut expected: Vec<usize> = (3..10).collect();
		expected.push(0);

		for (i, pth) in tree.paths.iter().enumerate() {
			let should_match = expected.contains(&i);
			assert_eq!(pth.borrow().matched, should_match);
		}
	}

	#[test]
	fn correct_tree_string() {
		let paths = create_test_paths();
		let mut tree = Tree::from_paths(paths);
		tree.filter("XX");
		let response = tree.as_lines();
		let expected: Vec<String> = vec![];
		assert_eq!(response, expected);
	}

	#[test]
	fn correct_n_descendants() {
		let paths = create_test_paths();
		let tree = create_test_tree(&paths);
		assert_eq!(tree.n_descendants(), 10);
		assert_eq!(paths[3].n_descendants(), 6);
	}

	#[test]
	fn reducing_patterns() {
		assert_eq!(reduce_patterns(&vec!["abc", "def"]), vec!["abc", "def"]);
		assert_eq!(reduce_patterns(&vec!["abc", "abc"]), vec!["abc"]);
		assert_eq!(reduce_patterns(&vec!["aaa", "aaaa", "a"]), vec!["aaaa"]);
		assert_eq!(
			reduce_patterns(&vec!["apa", "aaaa", "a"]),
			vec!["aaaa", "apa"]
		);
	}

	#[test]
	fn matchfn_sets_matched_field_correctly() {
		let paths = vec![
			Path::new("this/is/aaaa/paath.txt".to_string(), false),
			Path::new("this/is/aaaa/paath.txt".to_string(), false),
			Path::new("this/is/aaaa/file.ext".to_string(), false),
		];
		for p in &paths {
			p.borrow_mut().matched = false;
		}
		matchfn(&paths, &vec!["aaaa", "this", "paath.txt"]);
		assert!(paths[0].borrow().matched);
		assert!(paths[1].borrow().matched);
		assert!(!paths[2].borrow().matched);

		assert_eq!(Tree::from_paths(paths).calc_n_matches(), 2);
	}

	#[test]
	fn matchfn_colors_basename() {
		let paths = vec![
			Path::new("this/is/file.rs".to_string(), false),
			Path::new("this/is/fxiyle.xrs".to_string(), false),
		];

		matchfn(&paths, &vec!["file.rs"]);
		assert_eq!(
			paths[0].borrow().match_text,
			format!("{}file.rs{}", BLUE, RESET)
		);

		matchfn(&paths, &vec!["x", "y"]);
		assert_eq!(
			paths[1].borrow().match_text,
			format!(
				"f{}x{}i{}y{}le.{}x{}rs",
				BLUE, RESET, BLUE, RESET, BLUE, RESET
			)
		);
	}
}
