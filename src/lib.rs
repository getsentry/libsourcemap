#![recursion_limit = "1024"]

extern crate sourcemap;
extern crate memmap;
extern crate brotli2;
extern crate varinteger;

#[macro_use]
extern crate error_chain;

mod errors;
pub mod memdb;

pub use errors::{Error, ErrorKind, ChainErr, Result};
pub use unified::{View, Index, TokenMatch};

// unified interface
mod unified;

// cabi for unified interface
pub mod cabi;
