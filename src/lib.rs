#![recursion_limit = "1024"]

extern crate sourcemap;
extern crate memmap;
extern crate brotli2;
extern crate byteorder;
extern crate varinteger;

#[macro_use]
extern crate error_chain;

mod errors;
pub mod memidx;
pub mod memdb;
pub mod bitpacker;

pub use errors::{Error, ErrorKind, ChainErr, Result};
pub use unified::{View, Index, TokenMatch};

// unified interface
mod unified;

// cabi for unified interface
pub mod cabi;
