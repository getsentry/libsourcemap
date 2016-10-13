extern crate sourcemap;

mod memdb;
mod writer;
mod types;

pub use writer::serialize_map;
pub use memdb::MemDb;
