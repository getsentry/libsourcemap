use std::mem;
use std::slice;
use std::io;
use std::io::{Write, Seek, SeekFrom};

use sourcemap::SourceMap;

use types::{IndexItem, StringMarker, MapHead};

fn write_obj<T, W: Write>(w: &mut W, x: &T) -> io::Result<u32> {
    unsafe {
        let bytes : *const u8 = mem::transmute(x);
        let size = mem::size_of_val(x);
        try!(w.write_all(slice::from_raw_parts(bytes, size)));
        Ok(size as u32)
    }
}

fn write_slice<T, W: Write>(w: &mut W, x: &[T]) -> io::Result<u32> {
    unsafe {
        let bytes : *const u8 = mem::transmute(x.as_ptr());
        let size = mem::size_of::<T>() * x.len();
        try!(w.write_all(slice::from_raw_parts(bytes, size)));
        Ok(size as u32)
    }
}

/// Serializes a map into a given writer
pub fn serialize_map<W: Write+Seek>(sm: &SourceMap, mut w: W) -> io::Result<()> {
    let mut head = MapHead {
        index_size: sm.get_index_size() as u32,
        names_start: 0,
        names_count: sm.get_name_count(),
        sources_start: 0,
        sources_count: sm.get_source_count(),
    };

    // this will later be the information where to skip to for the TOCs
    let mut idx = try!(write_obj(&mut w, &head));

    // write the index
    for (line, col, token_id) in sm.index_iter() {
        let token = sm.get_token(token_id).unwrap();
        let raw = token.get_raw_token();
        let item = IndexItem {
            line: line,
            col: col,
            name_id: raw.name_id,
            src_id: raw.src_id,
        };
        idx += try!(write_obj(&mut w, &item));
    }

    // write names
    let mut names = vec![];
    for name in sm.names() {
        let to_write = name.as_bytes();
        try!(w.write_all(to_write));
        names.push(StringMarker {
            pos: idx,
            len: to_write.len() as u32,
        });
        idx += to_write.len() as u32;
    }

    // write sources
    let mut sources = vec![];
    for source_id in 0..sm.get_source_count() {
        let source = sm.get_source(source_id).unwrap();
        let to_write = source.as_bytes();
        try!(w.write_all(to_write));
        sources.push(StringMarker {
            pos: idx,
            len: to_write.len() as u32,
        });
        idx += to_write.len() as u32;
    }

    // write indexes
    head.names_start = idx;
    idx += try!(write_slice(&mut w, &names));
    head.sources_start = idx;
    try!(write_slice(&mut w, &sources));

    // write offsets
    try!(w.seek(SeekFrom::Start(0)));
    try!(write_obj(&mut w, &head));

    Ok(())
}
