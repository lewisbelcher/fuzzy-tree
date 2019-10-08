Fuzzy Tree
==========

A simple fuzzy finder written in Rust which displays results in tree format.


TODO
----

* Don't start a new command line at exit..
* Implement ctrl+{w,u,y,left,right}
* Refactor stdin matching into handlers on `Tui`?
* Implement directory collapsing/expanding
* Highlight matched text
* Improve matching to include regexes
* Cache match results as an LRU

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
