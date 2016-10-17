use std::io::{Read, Cursor};

use sourcemap::{SourceMap, SourceMapIndex};

use memdb::{MemDb, sourcemap_to_memdb};
use errors::Result;


enum MapRepr {
    Json(SourceMap),
    Mem(MemDb<'static>),
}

pub struct View {
    map: MapRepr,
}

pub struct Index {
    index: SourceMapIndex,
}

#[derive(Debug)]
pub struct TokenMatch<'a> {
    pub dst_line: u32,
    pub dst_col: u32,
    pub src_line: u32,
    pub src_col: u32,
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

    pub fn from_sourcemap(sm: SourceMap) -> Result<View> {
        Ok(View {
            map: MapRepr::Json(sm)
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
                        src_line: tok.get_src_line(),
                        src_col: tok.get_src_col(),
                        dst_line: tok.get_dst_line(),
                        dst_col: tok.get_dst_col(),
                        name: tok.get_name(),
                        src: tok.get_source(),
                        src_id: tok.get_raw_token().src_id,
                    });
                }
            },
            MapRepr::Mem(ref db) => {
                if let Some(tok) = db.lookup_token(line, col) {
                    return Some(TokenMatch {
                        src_line: tok.get_src_line(),
                        src_col: tok.get_src_col(),
                        dst_line: tok.get_dst_line(),
                        dst_col: tok.get_dst_col(),
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

    pub fn get_source(&self, src_id: u32) -> Option<&str> {
        match self.map {
            MapRepr::Json(ref sm) => sm.get_source(src_id),
            MapRepr::Mem(ref db) => db.get_source(src_id),
        }
    }

    pub fn get_source_count(&self) -> u32 {
        match self.map {
            MapRepr::Json(ref sm) => sm.get_source_count(),
            MapRepr::Mem(ref db) => db.get_source_count(),
        }
    }

    pub fn get_token_count(&self) -> u32 {
        match self.map {
            MapRepr::Json(ref sm) => sm.get_token_count(),
            MapRepr::Mem(ref db) => db.get_token_count(),
        }
    }

    pub fn get_token<'a>(&'a self, idx: u32) -> Option<TokenMatch<'a>> {
        match self.map {
            MapRepr::Json(ref sm) => {
                if let Some(tok) = sm.get_token(idx) {
                    return Some(TokenMatch {
                        src_line: tok.get_src_line(),
                        src_col: tok.get_src_col(),
                        dst_line: tok.get_dst_line(),
                        dst_col: tok.get_dst_col(),
                        name: tok.get_name(),
                        src: tok.get_source(),
                        src_id: tok.get_raw_token().src_id,
                    });
                }
            },
            MapRepr::Mem(ref db) => {
                if let Some(tok) = db.get_token(idx) {
                    return Some(TokenMatch {
                        src_line: tok.get_src_line(),
                        src_col: tok.get_src_col(),
                        dst_line: tok.get_dst_line(),
                        dst_col: tok.get_dst_col(),
                        name: tok.get_name(),
                        src: tok.get_source(),
                        src_id: tok.get_raw_token().src_id,
                    });
                }
            }
        }
        None
    }
}

impl Index {
    pub fn json_from_slice(buffer: &[u8]) -> Result<Index> {
        Ok(Index {
            index: try!(SourceMapIndex::from_slice(&buffer)),
        })
    }

    pub fn into_view(self) -> Result<View> {
        View::from_sourcemap(try!(self.index.flatten()))
    }

    pub fn can_flatten(&self) -> bool {
        for section in self.index.sections() {
            if let None = section.get_sourcemap() {
                return false;
            }
        }
        true
    }
}
