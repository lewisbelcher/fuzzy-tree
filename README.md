Fuzzy Tree
==========

A simple fuzzy finder written in Rust which displays results in tree format.


TODO
----

* Implement ctrl+{left,right}
* By default directories are collapsed if they have over 10(?) children?
* Implement using a config file for:
  - Setting default command line arguments
  - Setting control keys
* Improve matching to include regexes
* Improve stability of tui (maybe termion is a little unstable?)

Example tree (used in tests):

```
.
├── A
├── B
├── src
│   ├── bayes
│   │   ├── blend.c
│   │   └── rand.c
│   └── cakes
│       ├── a.c
│       └── b.c
└── x.txt
```
