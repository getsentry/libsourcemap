use std::str::from_utf8;
use std::mem;
use std::fmt;
use std::slice;
use std::io;
use std::io::{Write, Seek, SeekFrom};

use sourcemap::{RawToken, SourceMap};


#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct IndexItem {
    pub line: u32,
    pub col: u32,
    pub name_id: u32,
    pub src_id: u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct StringMarker {
    pub pos: u32,
    pub len: u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct MapHead {
    pub index_size: u32,
    pub names_start: u32,
    pub names_count: u32,
    pub sources_start: u32,
    pub sources_count: u32,
}

pub struct MemDb<'a> {
    index: &'a [IndexItem],
    buffer: &'a [u8],
    names: &'a [StringMarker],
    sources: &'a [StringMarker],
}

pub struct Token<'a> {
    db: &'a MemDb<'a>,
    raw: RawToken,
}


impl<'a> MemDb<'a> {

    pub fn new(buffer: &'a [u8]) -> MemDb<'a> {
        let head_size = mem::size_of::<MapHead>();
        let head : &MapHead = unsafe {
            mem::transmute(buffer[0..head_size].as_ptr())
        };

        let index_len = mem::size_of::<IndexItem>() * head.index_size as usize;
        let names_len = mem::size_of::<StringMarker>() * head.names_count as usize;
        let sources_len = mem::size_of::<StringMarker>() * head.sources_count as usize;

        let index : &[IndexItem] = unsafe {
            slice::from_raw_parts(
                mem::transmute(buffer[head_size..index_len].as_ptr()),
                head.index_size as usize
            )
        };

        let names : &[StringMarker] = unsafe {
            let off = head.names_start as usize;
            slice::from_raw_parts(
                mem::transmute(buffer[off..off + names_len].as_ptr()),
                head.names_count as usize
            )
        };

        let sources : &[StringMarker] = unsafe {
            let off = head.sources_start as usize;
            slice::from_raw_parts(
                mem::transmute(buffer[off..off + sources_len].as_ptr()),
                head.sources_count as usize
            )
        };

        MemDb {
            index: index,
            buffer: buffer,
            names: names,
            sources: sources,
        }
    }

    pub fn lookup_token(&self, line: u32, col: u32) -> Option<Token> {
        let mut low = 0;
        let mut high = self.index.len();

        while low < high {
            let mid = (low + high) / 2;
            let ii = &self.index[mid as usize];
            if (line, col) < (ii.line, ii.col) {
                high = mid;
            } else {
                low = mid + 1;
            }
        }

        if low > 0 {
            let ii = &self.index[low as usize];
            Some(Token {
                db: self,
                raw: RawToken {
                    dst_line: line,
                    dst_col: col,
                    src_line: ii.line,
                    src_col: ii.col,
                    src_id: ii.src_id,
                    name_id: ii.name_id,
                }
            })
        } else {
            None
        }
    }

    pub fn get_source(&self, src_id: u32) -> Option<&str> {
        self.sources.get(src_id as usize).and_then(|marker| {
            let bytes = &self.buffer[
                marker.pos as usize..(marker.pos + marker.len) as usize];
            from_utf8(bytes).ok()
        })
    }

    pub fn get_name(&self, name_id: u32) -> Option<&str> {
        self.names.get(name_id as usize).and_then(|marker| {
            let bytes = &self.buffer[
                marker.pos as usize..(marker.pos + marker.len) as usize];
            from_utf8(bytes).ok()
        })
    }
}

impl<'a> Token<'a> {

    /// get the destination (minified) line number
    pub fn get_dst_line(&self) -> u32 {
        self.raw.dst_line
    }

    /// get the destination (minified) column number
    pub fn get_dst_col(&self) -> u32 {
        self.raw.dst_col
    }

    /// get the destination line and column
    pub fn get_dst(&self) -> (u32, u32) {
        (self.get_dst_line(), self.get_dst_col())
    }

    /// get the source line number
    pub fn get_src_line(&self) -> u32 {
        self.raw.src_line
    }

    /// get the source column number
    pub fn get_src_col(&self) -> u32 {
        self.raw.src_col
    }

    /// get the source line and column
    pub fn get_src(&self) -> (u32, u32) {
        (self.get_src_line(), self.get_src_col())
    }

    /// get the source if it exists as string
    pub fn get_source(&self) -> &str {
        self.db.get_source(self.raw.src_id).unwrap_or("")
    }

    /// get the name if it exists as string
    pub fn get_name(&self) -> Option<&str> {
        self.db.get_name(self.raw.src_id)
    }

    /// returns `true` if a name exists, `false` otherwise
    pub fn has_name(&self) -> bool {
        self.get_name().is_some()
    }

    /// Converts the token into a debug tuple in the form
    /// `(source, src_line, src_col, name)`
    pub fn to_tuple(&self) -> (&str, u32, u32, Option<&str>) {
        (
            self.get_source(),
            self.get_src_line(),
            self.get_src_col(),
            self.get_name()
        )
    }

    /// Get the underlying raw token
    pub fn get_raw_token(&self) -> RawToken {
        self.raw
    }
}

impl<'a> fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Token {}>", self)
    }
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}{}",
               self.get_source(),
               self.get_src_line(),
               self.get_src_col(),
               self.get_name().map(|x| format!(" name={}", x))
                   .unwrap_or("".into()))
    }
}

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
pub fn sourcemap_to_memdb<W: Write+Seek>(sm: &SourceMap, mut w: W) -> io::Result<()> {
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
