use std::fs;
use std::env;
use std::io::Read;

extern crate sourcemap;
extern crate sourcemap_db;

use sourcemap::SourceMap;
use sourcemap_db::{serialize_map, MemDb};

fn main() {
    let args : Vec<_> = env::args().collect();

    let f = fs::File::open(&args[1]).unwrap();
    let sm = SourceMap::from_reader(&f).unwrap();

    // write index
    {
        let db_out = fs::File::create("/tmp/index").unwrap();
        serialize_map(&sm, db_out).unwrap();
    }

    // load index
    {
        let mut db = fs::File::open("/tmp/index").unwrap();
        let mut buf = vec![];
        db.read_to_end(&mut buf).unwrap();
        let memdb = MemDb::new(&buf);
        println!("{:?}", memdb.lookup_token(10, 200));
    }
}
