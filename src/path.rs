use crate::tree::{self, Breeder};
use std::cmp::Ordering;
use std::path;
use std::rc::Rc;

pub type PathBranch<'a> = tree::XBranch<&'a Path>;

#[derive(Eq, PartialEq, Debug)]
pub struct Path {
	pub components: Vec<String>,
	pub selected: bool,
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
	pub fn new(string: String) -> Path {
		Path {
			components: string
				.split(path::MAIN_SEPARATOR)
				.map(|x| x.to_string())
				.collect(),
			selected: false,
		}
	}

	pub fn from(string: &str) -> Path {
		Path::new(string.to_string())
	}

	pub fn joined(&self) -> String {
		self.components.join(&path::MAIN_SEPARATOR.to_string())
	}

	pub fn basename(&self) -> &str {
		&self.components[self.components.len() - 1]
	}

	pub fn len(&self) -> usize {
		self.components.len()
	}

	pub fn is_parent_of(&self, other: &Path) -> bool {
		is_child_of(other, self)
	}

	pub fn is_child_of(&self, other: &Path) -> bool {
		is_child_of(self, other)
	}
}

fn is_child_of(p1: &Path, p2: &Path) -> bool {
	if p2.len() >= p1.len() {
		return false;
	}
	p1.components[..p2.len()] == p2.components[..]
}

/// Create multiple paths from a `find`-like command output.
pub fn create_paths(string: Vec<u8>) -> Vec<Path> {
	let mut paths: Vec<Path> = String::from_utf8(string)
		.unwrap()
		.split('\n')
		.filter(|x| !x.is_empty())
		.map(|x| Path::new(x.to_string()))
		.collect();

	paths.sort();

	// Add CWD as "."
	paths.insert(0, Path::new(".".to_string()));
	for p in &mut paths {
		p.components.insert(0, ".".to_string());
	}

	paths
}

fn _match(pth: &Path, pattern: &str) -> bool {
	let split: Vec<&str> = pattern.split(" ").filter(|x| !x.is_empty()).collect();
	let mut matched;

	for s in split {
		matched = false;
		for c in &pth.components {
			matched |= c.contains(s);
		}
		if !matched {
			return false;
		}
	}
	true
}

pub fn filter<'t>(paths: &'t [Path], pattern: &str) -> Vec<usize> {
	let mut i = 0;
	paths
		.iter()
		.filter_map(move |x| {
			let r = if _match(x, pattern) { Some(i) } else { None };
			i += 1;
			return r;
		})
		.collect()
}

fn _ischild(a: &PathBranch, b: &PathBranch) -> bool {
	a.borrow().elem.is_child_of(b.borrow().elem)
}

fn peek<'a>(stack: &'a [PathBranch<'a>], i: usize) -> Option<&'a PathBranch<'a>> {
	if i < stack.len() {
		return Some(&stack[i]);
	}
	None
}

macro_rules! debug_relation {
	($obj1:expr; child $obj2:expr) => {
		println!(
			"{:?} is child of {:?}",
			$obj1.borrow().elem,
			$obj2.borrow().elem
		);
	};
	($obj1:expr; unrelated $obj2:expr, $obj3:expr) => {
		println!(
			"{:?} is unrelated to {:?} and {:?}",
			$obj1.borrow().elem,
			$obj2.borrow().elem,
			$obj3.borrow().elem
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
fn recurse_tree<'a>(
	mut i: usize,
	base: &PathBranch<'a>,
	mut prev: Option<&PathBranch<'a>>,
	stack: &'a [PathBranch<'a>],
) -> usize {
	// TODO: Use log package
	loop {
		println!("~~~ {} ~~~", i);
		i += 1;
		if let Some(next) = peek(stack, i) {
			if let Some(prev) = prev {
				if _ischild(&next, &prev) {
					debug_relation!(next; child base);
					prev.add_child(&next);
					i = recurse_tree(i, &prev, Some(&next), stack) - 1;
				} else if _ischild(&next, &base) {
					debug_relation!(next; child base);
					base.add_child(&next);
				} else {
					debug_relation!(next; unrelated base, prev);
					break i;
				}
			} else if _ischild(&next, &base) {
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
pub fn create_tree<'a>(paths: &'a Vec<PathBranch<'a>>) -> PathBranch<'a> {
	recurse_tree(0, &paths[0], None, &paths[..]);
	Rc::clone(&paths[0])
}

#[macro_export]
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

#[cfg(test)]
mod test {
	use super::*;

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
	}

	#[test]
	fn sorting_is_correct_2() {
		let mut paths = paths!["src", "tmp", "src/main.rs"];
		paths.sort();
		let expected = paths!["src", "src/main.rs", "tmp"];
		assert_eq!(paths, expected);
	}

	#[test]
	fn filter_gives_correct_elements() {
		let paths = paths!["here/is/x/path.c", "here/is/y/path.c", "here/is/z/path.c"];

		assert_eq!(filter(&paths[..], "x"), vec![0]);
		assert_eq!(filter(&paths[..], "y"), vec![1]);
		assert_eq!(filter(&paths[..], "z"), vec![2]);
		assert_eq!(filter(&paths[..], "x y"), vec![]);
		assert_eq!(filter(&paths[..], "x h"), vec![0]);
		assert_eq!(filter(&paths[..], "here"), vec![0, 1, 2]);
	}

	#[test]
	fn is_child_of_correctness() {
		let mut p1 = Path::from("A");
		let mut p2 = Path::from("B");
		assert!(!is_child_of(&p1, &p2));

		p1 = Path::from("src/bayes");
		p2 = Path::from("src/bayes/blend.c");
		assert!(!is_child_of(&p1, &p2));

		p1 = Path::from("src/bayes");
		p2 = Path::from("src/bayes/blend.c");
		assert!(is_child_of(&p2, &p1));
	}

	/// Determine whether two trees are equal by recursing into all branches
	fn trees_equal(a: &PathBranch, b: &PathBranch) -> bool {
		if a.borrow().elem != b.borrow().elem {
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

	#[test]
	fn create_tree_correct() {
		let paths = paths![
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
		];
		let branches: Vec<PathBranch> = paths.iter().map(tree::Branch::new).collect();
		let root = create_tree(&branches);
		println!("{:?}", root);  // TODO: Use log

		let expected: Vec<PathBranch> = paths.iter().map(tree::Branch::new).collect();
		let root2 = Rc::clone(&expected[0]);
		root2.add_child(&expected[1]);
		root2.add_child(&expected[2]);
		root2.add_child(&expected[3]);
		&expected[3].add_child(&expected[4]);
		&expected[4].add_child(&expected[5]);
		&expected[4].add_child(&expected[6]);
		&expected[3].add_child(&expected[7]);
		&expected[7].add_child(&expected[8]);
		&expected[7].add_child(&expected[9]);
		root2.add_child(&expected[10]);

		assert!(trees_equal(&root, &root2));
	}
}
