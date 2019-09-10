use std::cmp::Ordering;
use std::path;

#[derive(Eq, PartialEq, Debug)]
pub struct Path {
	components: Vec<String>,
	pub selected: bool,
}

impl Ord for Path {
	fn cmp(&self, other: &Self) -> Ordering {
		let mut result;

		// First compare lengths
		result = self.components.len().cmp(&other.components.len());
		if result != Ordering::Equal {
			return result;
		}

		// Then compare contents
		for (x, y) in self.components.iter().zip(other.components.iter()) {
			result = x.cmp(y);
			if result != Ordering::Equal {
				return result;
			}
		}

		// Finally, they must be equal!
		Ordering::Equal
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

	fn print(&self) -> String {
		format!("{}", "    │".repeat(self.components.len() - 2))
	}
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
}
