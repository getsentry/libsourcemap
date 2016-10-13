#![recursion_limit = "1024"]

extern crate sourcemap;

#[macro_use]
extern crate error_chain;

mod errors;
pub mod memdb;

pub use errors::{Error, ErrorKind, ChainErr, Result};
pub use unified::{View, TokenMatch};

// unified interface
mod unified;

// cabi for unified interface
pub mod cabi;
