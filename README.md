Fuzzy Tree
==========

A simple fuzzy finder written in Rust which displays results in tree format.


TODO
----

* Print in tree format.
* Don't start a new command line at exit..
* Implement ctrl+{w,u,y,left,right}
* Refactor stdin matching into handlers on `Tui`?

```
.
├── A
├── Cargo.lock
├── Cargo.toml
├── README.md
├── src
│   ├── main.rs
│   ├── path.rs
│   └── tui.rs
└── tmp
    ├── 0.txt
    ├── 1.txt
    ├── 2.txt
    ├── 3.txt
    ├── 4.txt
    └── 5.txt
```


```
.
├── A
├── Cargo.lock
├── Cargo.toml
├── README.md
├── src
│   ├── main.rs
│   ├── path.rs
│   └── tui.rs
├── target
│   └── debug
│       ├── build
│       │   ├── libc-6d7e6fd31f121591
│       │   │   ├── build-script-build
│       │   │   ├── build_script_build-6d7e6fd31f121591
│       │   │   └── build_script_build-6d7e6fd31f121591.d
│       │   └── libc-9ebe22d30276a428
│       │       ├── invoked.timestamp
│       │       ├── out
│       │       ├── output
│       │       ├── root-output
│       │       └── stderr
```
