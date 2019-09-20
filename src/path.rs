use std::cmp::Ordering;
use std::path;

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

	pub fn joined(&self) -> String {
		self.components.join(&path::MAIN_SEPARATOR.to_string())
	}

	pub fn basename(&self) -> &str {
		&self.components[self.components.len() - 1]
	}

	pub fn len(&self) -> usize {
		self.components.len()
	}
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

	// paths.insert(0, Path::new("".to_string()));
	// for p in &mut paths {
	// 	p.components.insert(0, ".".to_string());
	// }

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

fn is_child_of(p1: &Path, p2: &Path) -> bool {
	if p2.len() >= p1.len() {
		return false;
	}
	p1.components[..p2.len()] == p2.components[..]
}

fn is_only_child(p: &Path, paths: &[Path]) -> bool {
	let len = p.components.len();
	let reference = &p.components[..len - 1];

	for pth in paths.iter() {
		if len == pth.components.len() && reference == &pth.components[..len - 1] {
			return false;
		}
	}
	true
}

fn in_same_dir(p1: &Path, p2: &Path) -> bool {
	p1.len() == p2.len() && p1.components[..p1.len() - 1] == p2.components[..p1.len() - 1]
}

fn create_dir(mut i: usize, prefix: &str, paths: &[Path], lines: &mut Vec<String>) -> usize {
	if i == paths.len() {
		return i;
	}

	let pth1 = &paths[i];

	let extra = if i < paths.len() - 1 && in_same_dir(pth1, &paths[i + 1]) {
		"├── "
	} else {
		"└── "
	};
	lines.push(prefix.to_owned() + extra + pth1.basename());

	i += 1;
	if i == paths.len() {
		return i;
	}

	let pth2 = &paths[i];

	if is_child_of(pth2, pth1) {
		let extra = if i < paths.len() - 1 && is_only_child(pth2, &paths[i + 1..]) {
			"    "
		} else {
			"│   "
		};
		return create_dir(i, &(prefix.to_owned() + extra), paths, lines);
	} else {
		let diff = prefix.len() - (4 * (pth1.len() - pth2.len()));
		return create_dir(i, &prefix[..diff], paths, lines);
	}
}

/// Create a vector of strings representing lines in the tree printout.
///
/// TODO: I think this needs to be a recursive function!
// pub fn create_tree() {}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn sorting_is_correct() {
		let mut path1;
		let mut path2;

		path1 = Path::new("here/is/a/path.c".to_string());
		path2 = Path::new("here/is/a/path.c".to_string());
		assert_eq!(path1, path2);

		path1 = Path::new("here/is/a".to_string());
		path2 = Path::new("here/is/a/path.c".to_string());
		assert!(path1 < path2);

		path1 = Path::new("here/is/a/fath.c".to_string());
		path2 = Path::new("here/is/a/path.c".to_string());
		assert!(path1 < path2);
	}

	#[test]
	fn sorting_is_correct_2() {
		let mut paths = vec![
			Path::new("src".to_string()),
			Path::new("tmp".to_string()),
			Path::new("src/main.rs".to_string()),
		];
		paths.sort();
		let expected = vec![
			Path::new("src".to_string()),
			Path::new("src/main.rs".to_string()),
			Path::new("tmp".to_string()),
		];
		assert_eq!(paths, expected);
	}

	#[test]
	fn filter_gives_correct_elements() {
		let paths = vec![
			Path::new("here/is/x/path.c".to_string()),
			Path::new("here/is/y/path.c".to_string()),
			Path::new("here/is/z/path.c".to_string()),
		];

		assert_eq!(filter(&paths[..], "x"), vec![0]);
		assert_eq!(filter(&paths[..], "y"), vec![1]);
		assert_eq!(filter(&paths[..], "z"), vec![2]);
		assert_eq!(filter(&paths[..], "x y"), vec![]);
		assert_eq!(filter(&paths[..], "x h"), vec![0]);
		assert_eq!(filter(&paths[..], "here"), vec![0, 1, 2]);
	}

	#[test]
	fn is_child_of_correctness() {
		let mut p1 = Path::new("A".to_string());
		let mut p2 = Path::new("B".to_string());
		assert!(!is_child_of(&p1, &p2));

		p1 = Path::new("src/bayes".to_string());
		p2 = Path::new("src/bayes/blend.c".to_string());
		assert!(!is_child_of(&p1, &p2));

		p1 = Path::new("src/bayes".to_string());
		p2 = Path::new("src/bayes/blend.c".to_string());
		assert!(is_child_of(&p2, &p1));
	}

	#[test]
	fn is_only_child_test() {
		let p = Path::new("src/bayes".to_string());
		let mut paths = vec![
			Path::new("src/bayes/blend.c".to_string()),
			Path::new("src/bayes/rand.c".to_string()),
			Path::new("x.txt".to_string()),
		];
		assert!(is_only_child(&p, &paths[..]));

		paths = vec![
			Path::new("src/bayes/blend.c".to_string()),
			Path::new("src/bayes/rand.c".to_string()),
			Path::new("src/cakes".to_string()),
			Path::new("x.txt".to_string()),
		];
		assert!(!is_only_child(&p, &paths[..]));
	}

	#[test]
	fn create_dir_correct() {
		let mut lines = Vec::new();
		let paths = vec![
			Path::new("A".to_string()),
			Path::new("B".to_string()),
			Path::new("src".to_string()),
			Path::new("src/bayes".to_string()),
			Path::new("src/bayes/blend.c".to_string()),
			Path::new("src/bayes/rand.c".to_string()),
			Path::new("x.txt".to_string()),
		];
		let response = create_dir(0, "", &paths[..], &mut lines);
		println!("{}", lines.join("\n"));
	}
}
