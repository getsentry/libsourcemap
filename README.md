# libsourcemap

This project implements efficient sourcemap handling in Rust and wraps
it for Python.  It's based on [rust-sourcemap](https://github.com/mitsuhiko/rust-sourcemap)
for the main sourcemap handling and implements a separate format that
can be cached more efficiently.

Tested with Rust 1.10 and later.

## Development

This requires a recent version of pip, setuptools and Rust to build.  To
get rust use [rustup](https://rustup.rs/).

To compile extension and wrappers use this with an enabled virtualenv:

```
make develop
```
