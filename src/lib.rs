extern crate sourcemap;

// in memory sourcemap support
mod memdb;
pub use memdb::{MemDb, Token, sourcemap_to_memdb};
