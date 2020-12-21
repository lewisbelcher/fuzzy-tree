Fuzzy Tree
==========

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![pipeline](https://gitlab.com/lewisbelcher/fuzzy-tree/badges/master/pipeline.svg)](https://gitlab.com/lewisbelcher/fuzzy-tree/pipelines)
[![crate](https://img.shields.io/crates/v/fuzzy-tree.svg)](https://crates.io/crates/fuzzy-tree)

A simple fuzzy finder written in Rust which displays results in an interactive
tree format.

![Fuzzy Tree gif](https://gitlab.com/lewisbelcher/fuzzy-tree/-/raw/master/static/fztree.gif)


Install
-------

1. [Get Rust](https://www.rust-lang.org/tools/install)
2. Install [`fd`](https://crates.io/crates/fd-find) (optional but recommended,
   the default find command is `fd`)
3. Clone this repo (optional)
4. Run `cargo install --path <repo path>` (if you did step 3) or `cargo install
   fuzzy-tree`
5. Use `fztree`!


Usage
=====

Using a different command (`--cmd`): The output from the command given must be
a new-line separated list of strings. While running fuzzy tree on a file list
generated from elsewhere, the directory prefix and functionality currently
relies on input items being detectable as directories *relative to the path
fuzzy tree was executed from.* (This could be mitigated by utilising a trailing
separator, `/` or `\`, to detect directories. Currently neither `find` nor `fd`
make this easily available however).


Contributing
------------

Contributions are very welcome! Feel free to fork and open MRs (PRs in GitHub
speak). NB the [GitLab site](https://gitlab.com/lewisbelcher/fuzzy-tree) is
the primary location for this project, so issues/MRs should be opened there.

My only preferences are:
* A relevant issue is opened and discussed.
* MRs are concise.
* Code is formatted according to local `rustfmt` rules.
* Tests are implemented/updated.
