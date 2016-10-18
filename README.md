# libsourcemap

This project implements efficient sourcemap handling in Rust and wraps
it for Python.  It's based on [rust-sourcemap](https://github.com/mitsuhiko/rust-sourcemap)
for the main sourcemap handling and implements a separate format that
can be cached more efficiently.

Tested with Rust 1.12 and later.

## Development

This requires a recent version of pip, setuptools and Rust to build.  To
get rust use [rustup](https://rustup.rs/).

To compile extension and wrappers use this with an enabled virtualenv:

```
make develop
```

## Speed Benchmark

This is a benchmark against one of the sentry customer files that comes in
at 30MB in size:

```
parse_json ... 516.52ms
  lookup_token ... 0.05ms
dump_memdb ... 217.95ms
load_memdb ... 50.87ms
  lookup_token ... 0.02ms
load_mmap ... 0.05ms
  lookup_token ... 0.03ms
```

Unexpectedly parsing is the slowest followed by dumping out an index in the
memdb format.  The latter can most likely be optimized.  Looking up tokens
is fast in most formats but fastest with memdb.  Loading memdb via mmap gives
the best loading time if the file to load is coming from the file system
with a slight performance hit on actually looking up symbols.
