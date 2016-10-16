use std::str::from_utf8;
use std::mem;
use std::fmt;
use std::slice;
use std::io;
use std::io::{Write, Seek, SeekFrom};
use std::borrow::Cow;

use sourcemap::{RawToken, SourceMap};

use errors::{ErrorKind, Result};


#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct IndexItem {
    pub dst_line: u32,
    pub dst_col: u32,
    pub src_line: u16,
    pub src_col: u16,
    pub ids: u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct MapHead {
    pub version: u32,
    pub index_size: u32,
    pub names_start: u32,
    pub names_count: u32,
    pub sources_start: u32,
    pub sources_count: u32,
    pub source_contents_start: u32,
    pub source_contents_count: u32,
}

pub struct MemDb<'a> {
    buffer: Cow<'a, [u8]>,
}

pub struct Token<'a> {
    db: &'a MemDb<'a>,
    raw: RawToken,
}


impl<'a> MemDb<'a> {

    pub fn from_cow(cow: Cow<'a, [u8]>) -> Result<MemDb<'a>> {
        let rv = MemDb {
            buffer: cow,
        };
        if try!(rv.header()).version != 1 {
            Err(ErrorKind::UnsupportedMemDbVersion.into())
        } else {
            Ok(rv)
        }
    }

    pub fn from_slice(buffer: &'a [u8]) -> Result<MemDb<'a>> {
        MemDb::from_cow(Cow::Borrowed(buffer))
    }

    pub fn from_vec(buffer: Vec<u8>) -> Result<MemDb<'a>> {
        MemDb::from_cow(Cow::Owned(buffer))
    }

    pub fn get_name(&self, name_id: u32) -> Option<&str> {
        self.names().ok().and_then(|x| self.get_string(x, name_id))
    }

    pub fn get_source(&self, src_id: u32) -> Option<&str> {
        self.sources().ok().and_then(|x| self.get_string(x, src_id))
    }

    pub fn get_source_contents(&self, src_id: u32) -> Option<&str> {
        self.source_contents().ok().and_then(|x| self.get_string(x, src_id))
    }

    pub fn get_token_count(&self) -> u32 {
        self.header().map(|x| x.index_size).unwrap_or(0)
    }

    pub fn get_token(&'a self, idx: u32) -> Option<Token<'a>> {
        self.index().ok().and_then(|index| {
            (&index.get(idx as usize)).map(|ii| {
                Token {
                    db: self,
                    raw: RawToken {
                        dst_line: ii.dst_line,
                        dst_col: ii.dst_col,
                        src_line: ii.src_line as u32,
                        src_col: ii.src_col as u32,
                        src_id: ii.ids >> 22,
                        name_id: ii.ids & 0x3fffff,
                    }
                }
            })
        })
    }

    pub fn lookup_token(&'a self, line: u32, col: u32) -> Option<Token<'a>> {
        let index = match self.index() {
            Ok(idx) => idx,
            Err(_) => { return None; }
        };
        let mut low = 0;
        let mut high = index.len();

        while low < high {
            let mid = (low + high) / 2;
            let ii = &index[mid as usize];
            if (line, col) < (ii.dst_line, ii.dst_col) {
                high = mid;
            } else {
                low = mid + 1;
            }
        }

        if low < high {
            self.get_token(low as u32)
        } else {
            None
        }
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    fn get_data(&self, start: usize, len: usize) -> Result<&[u8]> {
        let end = start.wrapping_add(len);
        if end < start || end > self.buffer.len() {
            Err(ErrorKind::BadMemDb.into())
        } else {
            Ok(&self.buffer[start..end])
        }
    }

    fn get_slice<T>(&self, offset: usize, count: usize) -> Result<&[T]> {
        let size = mem::size_of::<T>();
        Ok(unsafe {
            slice::from_raw_parts(
                mem::transmute(try!(self.get_data(offset, count * size)).as_ptr()),
                count
            )
        })
    }

    fn get_string(&self, coll: &[u32], idx: u32) -> Option<&str> {
        coll.get(idx as usize).and_then(|offset| {
            let offset = *offset as usize;
            let buffer = self.buffer();
            let mut len = 0;
            loop {
                if offset + len >= buffer.len() {
                    return None;
                } else if buffer[offset + len] == b'\x00' {
                    break;
                }
                len += 1;
            }
            from_utf8(&buffer[offset..offset + len]).ok()
        })
    }

    #[inline(always)]
    fn header(&self) -> Result<&MapHead> {
        unsafe {
            Ok(mem::transmute(try!(self.get_data(0, mem::size_of::<MapHead>())).as_ptr()))
        }
    }

    #[inline(always)]
    fn index(&self) -> Result<&[IndexItem]> {
        let head = try!(self.header());
        let off = mem::size_of::<MapHead>();
        self.get_slice(off, head.index_size as usize)
    }

    #[inline(always)]
    fn names(&self) -> Result<&[u32]> {
        let head = try!(self.header());
        let off = head.names_start as usize;
        self.get_slice(off, head.names_count as usize)
    }

    #[inline(always)]
    fn sources(&self) -> Result<&[u32]> {
        let head = try!(self.header());
        let off = head.sources_start as usize;
        self.get_slice(off, head.sources_count as usize)
    }

    #[inline(always)]
    fn source_contents(&self) -> Result<&[u32]> {
        let head = try!(self.header());
        let off = head.source_contents_start as usize;
        self.get_slice(off, head.source_contents_count as usize)
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
    pub fn get_source(&self) -> &'a str {
        self.db.get_source(self.raw.src_id).unwrap_or("")
    }

    /// get the name if it exists as string
    pub fn get_name(&self) -> Option<&'a str> {
        self.db.get_name(self.raw.name_id)
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

fn write_cstr<W: Write>(w: &mut W, bytes: &[u8]) -> io::Result<u32> {
    // XXX: strip out contained null bytes? does not seem to cause a security
    // problem here so maybe we can live with not doing that for now.
    try!(w.write_all(bytes));
    try!(w.write_all(b"\x00"));
    Ok(bytes.len() as u32 + 1)
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
        version: 1,
        index_size: sm.get_index_size() as u32,
        names_start: 0,
        names_count: sm.get_name_count(),
        sources_start: 0,
        sources_count: sm.get_source_count(),
        source_contents_start: 0,
        source_contents_count: 0,
    };

    // this will later be the information where to skip to for the TOCs
    let mut idx = try!(write_obj(&mut w, &head));

    // write the index
    for (line, col, token_id) in sm.index_iter() {
        let token = sm.get_token(token_id).unwrap();
        let raw = token.get_raw_token();
        assert!(line == raw.dst_line);
        assert!(col == raw.dst_col);
        let item = IndexItem {
            dst_line: line,
            dst_col: col,
            src_line: raw.src_line as u16,
            src_col: raw.src_col as u16,
            // XXX: these can individually overflow
            ids: (raw.src_id << 22 | raw.name_id),
        };
        idx += try!(write_obj(&mut w, &item));
    }

    // write names
    let mut names = vec![];
    for name in sm.names() {
        names.push(idx);
        idx += try!(write_cstr(&mut w, name.as_bytes()));
    }

    // write sources
    let mut sources = vec![];
    let mut source_contents = vec![];
    let mut have_sources = false;
    for source_id in 0..sm.get_source_count() {
        let source = sm.get_source(source_id).unwrap();
        sources.push(idx);
        idx += try!(write_cstr(&mut w, source.as_bytes()));

        if let Some(contents) = sm.get_source_contents(source_id) {
            have_sources = true;
            source_contents.push(idx);
            idx += try!(write_cstr(&mut w, contents.as_bytes()));
        } else {
            source_contents.push(!0);
        }
    }

    // write indexes
    head.names_start = idx;
    idx += try!(write_slice(&mut w, &names));
    head.sources_start = idx;
    idx += try!(write_slice(&mut w, &sources));

    if have_sources {
        head.source_contents_start = idx;
        head.source_contents_count = source_contents.len() as u32;
        try!(write_slice(&mut w, &source_contents));
    }

    // write offsets
    try!(w.seek(SeekFrom::Start(0)));
    try!(write_obj(&mut w, &head));

    Ok(())
}
