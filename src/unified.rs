use std::io::{Read, Cursor};

use sourcemap::SourceMap;

use memdb::{MemDb, sourcemap_to_memdb};
use errors::Result;


enum MapRepr {
    Json(SourceMap),
    Mem(MemDb<'static>),
}

pub struct View {
    map: MapRepr,
}

#[derive(Debug)]
pub struct TokenMatch<'a> {
    pub line: u32,
    pub col: u32,
    pub name: Option<&'a str>,
    pub src: &'a str,
    pub src_id: u32,
}

impl View {
    pub fn json_from_slice(buffer: &[u8]) -> Result<View> {
        Ok(View {
            map: MapRepr::Json(try!(SourceMap::from_slice(&buffer)))
        })
    }

    pub fn json_from_reader<R: Read>(rdr: R) -> Result<View> {
        Ok(View {
            map: MapRepr::Json(try!(SourceMap::from_reader(rdr)))
        })
    }

    pub fn memdb_from_vec(vec: Vec<u8>) -> Result<View> {
        Ok(View {
            map: MapRepr::Mem(try!(MemDb::from_vec(vec)))
        })
    }

    pub fn dump_memdb(&self) -> Vec<u8> {
        match self.map {
            MapRepr::Json(ref sm) => {
                let mut rv = Cursor::new(vec![]);
                // this should not fail if we write to memory
                sourcemap_to_memdb(sm, &mut rv).unwrap();
                rv.into_inner()
            },
            MapRepr::Mem(ref db) => db.buffer().to_vec()
        }
    }

    pub fn lookup_token<'a>(&'a self, line: u32, col: u32) -> Option<TokenMatch<'a>> {
        match self.map {
            MapRepr::Json(ref sm) => {
                if let Some(tok) = sm.lookup_token(line, col) {
                    return Some(TokenMatch {
                        line: tok.get_src_line(),
                        col: tok.get_src_col(),
                        name: tok.get_name(),
                        src: tok.get_source(),
                        src_id: tok.get_raw_token().src_id,
                    });
                }
            },
            MapRepr::Mem(ref db) => {
                if let Some(tok) = db.lookup_token(line, col) {
                    return Some(TokenMatch {
                        line: tok.get_src_line(),
                        col: tok.get_src_col(),
                        name: tok.get_name(),
                        src: tok.get_source(),
                        src_id: tok.get_raw_token().src_id,
                    });
                }
            }
        }
        None
    }

    pub fn get_source_contents(&self, src_id: u32) -> Option<&str> {
        match self.map {
            MapRepr::Json(ref sm) => sm.get_source_contents(src_id),
            MapRepr::Mem(ref db) => db.get_source_contents(src_id),
        }
    }
}
