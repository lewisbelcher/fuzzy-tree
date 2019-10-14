Fuzzy Tree
==========

A simple fuzzy finder written in Rust which displays results in tree format.


TODO
----

* Don't start a newline at exit..
* Implement ctrl+{w,u,y,left,right}
* Implement directory collapsing/expanding (use \` for this?)
* By default directories are collapsed if they have over 10(?) children?
* Allow setting find command / control keys via env vars / config file
* Allow setting config file location via env var
* Highlight matched text
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
