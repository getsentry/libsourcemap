use std::path::Path;
use std::fs;

extern crate sourcemap;
extern crate redis;

use redis::Commands;

pub fn demo(path: &Path) {
    let f = fs::File::open(path).unwrap();
    let sm = sourcemap::SourceMap::from_reader(&f).unwrap();

    let client = redis::Client::open("redis://127.0.0.1:12345").unwrap();
    let con = client.get_connection().unwrap();

    for (line, col, token_id) in sm.index_iter() {
        let token = sm.get_token(token_id).unwrap();
        let _ : () = con.zadd("index",
            format!("{}:{}:{}:{}",
                    token.get_src_line(),
                    token.get_src_col(),
                    token.get_source_id().unwrap(),
                    token.get_name().unwrap_or("")),
            format!("{}.{:010}", line, col)).unwrap();
    }

    for source_id in 0..sm.get_source_count() {
        let source = sm.get_source(source_id).unwrap();
        let _ : () = con.hset("index-sources", source_id, source).unwrap();
    }

    println!("{}", sm.get_source(0).unwrap());
}
