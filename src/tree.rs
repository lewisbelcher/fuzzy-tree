use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

pub type XBranch<T> = Rc<RefCell<Branch<T>>>;

pub struct Branch<T> {
	parent: Option<XBranch<T>>,
	pub elem: T,
	children: Option<Vec<XBranch<T>>>,
}

impl<T> fmt::Debug for Branch<T>
where
	T: std::fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}; Kids:", self.elem);
		if let Some(children) = &self.children {
			for ch in children {
				write!(f, "\n  {:?}", ch);
			}
		}
		write!(f, "")
	}
}

impl<T> Branch<T> {
	pub fn new(elem: T) -> Rc<RefCell<Self>> {
		Rc::new(RefCell::new(Branch {
			parent: None,
			elem,
			children: None,
		}))
	}

	pub fn depth(&self) -> usize {
		1 + match &self.parent {
			Some(p) => p.borrow().depth(),
			None => 0,
		}
	}
}

fn add<T>(child: &XBranch<T>, parent: &XBranch<T>) {
	let mut children = match parent.borrow_mut().children.take() {
		Some(v) => v,
		None => Vec::new(),
	};
	children.push(Rc::clone(child));
	parent.borrow_mut().children = Some(children);
	child.borrow_mut().parent = Some(Rc::clone(parent));
}

pub trait Breeder<T> {
	fn add_child(&self, child: &XBranch<T>);
	fn add_parent(&self, parent: &XBranch<T>);
}

impl<T> Breeder<T> for XBranch<T> {
	fn add_child(&self, child: &XBranch<T>) {
		add(child, self);
	}

	fn add_parent(&self, parent: &XBranch<T>) {
		add(self, parent);
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn add_some_depth() {
		let a = Branch::new("a");
		let b = Branch::new("b");
		let c = Branch::new("c");
		let d = Branch::new("d");
		a.add_child(&b);
		b.add_child(&c);
		d.add_parent(&c);
		assert_eq!(a.borrow().depth(), 1);
		assert_eq!(b.borrow().depth(), 2);
		assert_eq!(c.borrow().depth(), 3);
		assert_eq!(d.borrow().depth(), 4);
	}
}
