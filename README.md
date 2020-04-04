Fuzzy Tree
==========

A simple fuzzy finder written in Rust which displays results in tree format.


TODO
----

* Implement ctrl+{left,right} when supported (https://gitlab.redox-os.org/redox-os/termion/issues/46)
* Implement using a config file for:
  - Setting default command line arguments
  - Setting control keys
* Improve matching to include regexes
* Improve stability of tui

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
