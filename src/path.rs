use std::cmp::Ordering;
use std::path;

#[derive(Eq, PartialEq, Debug)]
pub struct Path {
	components: Vec<String>,
	pub selected: bool,
}

// TODO: Need to group directories with their sub-paths!
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

	pub fn print_joined(&self) {
		// self.components.join(&path::MAIN_SEPARATOR.to_string())
		print!(
			"{} ",
			self.components.join(&path::MAIN_SEPARATOR.to_string())
		);
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
}
