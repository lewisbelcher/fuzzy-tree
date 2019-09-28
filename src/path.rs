use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;
use std::path;
use std::rc::Rc;
use termion::color;

pub type RcPath = Rc<RefCell<Path>>;

lazy_static! {
	static ref SELECTED: String =
		format!("{}>{}", color::Fg(color::LightRed), color::Fg(color::Reset));
}

#[derive(Eq, PartialEq)]
pub struct Path {
	parent: Option<RcPath>,
	components: Vec<String>,
	pub selected: bool,
	matched: bool,
	children: Option<Vec<RcPath>>,
}

impl fmt::Debug for Path {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}; Children {:#?}:", self.components, self.children)
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
	pub fn new(string: String) -> RcPath {
		Rc::new(RefCell::new(Path {
			parent: None,
			components: string
				.split(path::MAIN_SEPARATOR)
				.map(|x| x.to_string())
				.collect(),
			selected: false,
			matched: true,
			children: None,
		}))
	}

	pub fn from(string: &str) -> RcPath {
		Path::new(string.to_string())
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

/// Since Paths are wrapped in Rc's which cannot be implemented on directly,
/// functionality has to be implemented via trait. These could be split into
/// logical groups of behaviours.
pub trait PathBehaviour {
	fn add_child(&self, child: &RcPath);
	fn add_parent(&self, parent: &RcPath);
	fn is_child_of(&self, other: &RcPath) -> bool;
	fn joined(&self) -> String;
	fn basename(&self) -> &str;
	fn len(&self) -> usize;
}

impl PathBehaviour for RcPath {
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

	fn joined(&self) -> String {
		self.borrow()
			.components
			.join(&path::MAIN_SEPARATOR.to_string())
	}

	fn basename(&self) -> &str {
		// TODO: See if there is a safe way around this:
		unsafe { &(*self.as_ptr()).components[self.len() - 1] }
	}

	fn len(&self) -> usize {
		self.borrow().components.len()
	}
}

/// Get the nth *matched* path from a vec of paths
pub fn get_n(paths: &Vec<RcPath>, i: usize) -> Option<&RcPath> {
	let mut matches = 0;
	for pth in paths {
		if pth.borrow().matched {
			if matches == i {
				return Some(pth);
			}
			matches += 1;
		}
	}
	None
}

fn _match(pth: &RcPath, pattern: &str) -> bool {
	let split: Vec<&str> = pattern.split(" ").filter(|x| !x.is_empty()).collect();
	let mut matched;

	for s in split {
		matched = false;
		for c in &pth.borrow().components {
			matched |= c.contains(s);
		}
		if !matched {
			return false;
		}
	}
	true
}

/// Update which paths are matched by `pattern`.
pub fn update_matched<'t>(paths: &'t Vec<RcPath>, pattern: &str) -> usize {
	let mut matched = 0;
	for pth in paths {
		if _match(pth, pattern) {
			pth.borrow_mut().matched = true;
			matched += 1;
		} else {
			pth.borrow_mut().matched = false;
		}
	}
	matched
}

/// Create multiple paths from a `find`-like command output.
pub fn create_paths(string: Vec<u8>) -> Vec<RcPath> {
	let mut paths: Vec<RcPath> = String::from_utf8(string)
		.unwrap()
		.split('\n')
		.filter(|x| !x.is_empty())
		.map(|x| Path::from(x))
		.collect();

	paths.sort();

	// Add CWD as "."
	for p in &mut paths {
		p.borrow_mut().components.insert(0, ".".to_string());
	}
	paths.insert(0, Path::from("."));

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
		println!("{:?} is child of {:?}", $obj1.borrow(), $obj2.borrow());
	};
	($obj1:expr; unrelated $obj2:expr, $obj3:expr) => {
		println!(
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
		println!("~~~ {} ~~~", i);
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

/// Inner recursive function to create a string representation of a directory
/// tree.
fn _tree_string(root: &RcPath, lines: &mut Vec<String>, prefix: &str) {
	if let Some(children) = &root.borrow().children {
		let children: Vec<&RcPath> = children.iter().filter(|x| x.borrow().matched).collect();
		for (i, child) in children.iter().enumerate() {
			let (pre, addon) = if i == children.len() - 1 {
				("    ", "└── ")
			} else {
				("│   ", "├── ")
			};

			let sel = if root.borrow().selected {
				&SELECTED
			} else {
				" "
			};
			lines.push(sel.to_owned() + prefix + addon + child.basename());
			if child.borrow().children.is_some() {
				_tree_string(child, lines, &(prefix.to_owned() + pre));
			}
		}
	} else {
		let sel = if root.borrow().selected {
			&SELECTED
		} else {
			" "
		};
		lines.push(sel.to_owned() + prefix + root.basename());
	}
}

/// Create a string representation of the tree from root directory `root`.
/// We can preallocate the exact capacity by knowing the number of paths
/// we are constructing for.
pub fn tree_string(root: &RcPath, len: usize) -> Vec<String> {
	let mut lines = Vec::with_capacity(len);
	_tree_string(root, &mut lines, "");
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
					temp.push(Path::from($x));
				)*
				temp
			}
		};
	}

	#[test]
	fn sorting_is_correct() {
		let mut path1;
		let mut path2;

		path1 = Path::from("here/is/a/path.c");
		path2 = Path::from("here/is/a/path.c");
		assert_eq!(path1, path2);

		path1 = Path::from("here/is/a");
		path2 = Path::from("here/is/a/path.c");
		assert!(path1 < path2);

		path1 = Path::from("here/is/a/fath.c");
		path2 = Path::from("here/is/a/path.c");
		assert!(path1 < path2);

		let mut paths = paths!["src", "tmp", "src/main.rs"];
		paths.sort();
		let expected = paths!["src", "src/main.rs", "tmp"];
		assert_eq!(paths, expected);
	}

	#[test]
	fn len_correct() {
		let s = "here/is/a/path.c";
		let path = Path::from(s);
		assert_eq!(path.len(), 4);
	}

	#[test]
	fn basename_correct() {
		let s = "here/is/a/path.c";
		let path = Path::from(s);
		assert_eq!(path.basename(), "path.c");
	}

	#[test]
	fn joined_joins_components() {
		let s = "here/is/a/path.c";
		let path = Path::from(s);
		assert_eq!(path.joined(), s);
	}

	#[test]
	fn is_child_of_correctness() {
		let mut p1 = Path::from("A");
		let mut p2 = Path::from("B");
		assert!(!p1.is_child_of(&p2));

		p1 = Path::from("src/bayes");
		p2 = Path::from("src/bayes/blend.c");
		assert!(!p1.is_child_of(&p2));

		p1 = Path::from("src/bayes");
		p2 = Path::from("src/bayes/blend.c");
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
		let root = create_tree(&paths);
		let root2 = create_test_tree(&paths);
		assert!(trees_equal(&root, &root2));
	}

	#[test]
	fn tree_string_test() {
		let mut paths = create_test_paths();
		let root = create_test_tree(&paths);
		let lines = tree_string(&root, paths.len());
		let expected = " ├── A
 ├── B
 ├── src
 │   ├── bayes
 │   │   ├── blend.c
 │   │   └── rand.c
 │   └── cakes
 │       ├── a.c
 │       └── b.c
 └── x.txt";
		assert_eq!(lines.join("\n"), expected);

		// Deselect `./src/bayes` and print again
		paths[4].borrow_mut().matched = false;
		let lines = tree_string(&root, paths.len());
		let expected = " ├── A
 ├── B
 ├── src
 │   └── cakes
 │       ├── a.c
 │       └── b.c
 └── x.txt";
		assert_eq!(lines.join("\n"), expected);
	}
}
