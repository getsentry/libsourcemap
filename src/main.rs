use std::path::Path;
use std::env;

extern crate sourcemap_db;

fn main() {
    let args : Vec<_> = env::args().collect();
    sourcemap_db::demo(Path::new(&args[1]));
}
