Fuzzy Tree
==========

A simple fuzzy finder written in Rust which displays results in tree format.


TODO
----

* Implement ctrl+{left,right}
* By default directories are collapsed if they have over 10(?) children?
* Allow setting find command / control keys via env vars / config file
* Allow setting config file location via env var
* Improve matching to include regexes

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
