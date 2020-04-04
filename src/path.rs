use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;
use std::fs;
use std::path;
use std::rc::Rc;

pub type RcPath = Rc<RefCell<Path>>;

#[derive(Eq, PartialEq)]
pub struct Path {
	pub components: Vec<String>,
	pub parent: Option<RcPath>,
	pub children: Option<Vec<RcPath>>,
	pub is_dir: bool,
	pub open: bool,
	pub matched: bool,
	pub match_text: String,
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
		return self.joined.cmp(&other.joined);
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
	fn flip_open(&self);
	fn add_child(&self, child: &RcPath);
	fn add_parent(&self, parent: &RcPath);
	fn is_child_of(&self, other: &RcPath) -> bool;
	fn basename(&self) -> &str;
	fn len(&self) -> usize;
	fn n_children(&self) -> usize;
	fn n_descendants(&self) -> usize;
}

impl PathBehaviour for RcPath {
	// TODO: Also add helper methods for borrow of `matched`, `selected`, `children`
	// etc.. This could be done with a macro?

	fn flip_open(&self) {
		let mut p = self.borrow_mut();
		p.open = !p.open;
	}

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

	fn n_children(&self) -> usize {
		if let Some(children) = &self.borrow().children {
			children.len()
		} else {
			0
		}
	}

	/// Total number of descendants of a path
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

#[macro_export]
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

#[cfg(test)]
mod test {
	use super::*;

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
}
