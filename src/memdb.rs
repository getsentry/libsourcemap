use std::str::from_utf8;
use std::mem;
use std::fmt;
use std::slice;
use std::io;
use std::path::Path;
use std::ptr::copy_nonoverlapping;
use std::io::{Read, Write, Seek, SeekFrom};
use std::borrow::Cow;
use memmap::{Mmap, Protection};

use varinteger;
use sourcemap::{RawToken, SourceMap};
use brotli2::read::{BrotliEncoder, BrotliDecoder};

use errors::{ErrorKind, Result};


#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct LargeIndexItem {
    pub packed_locinfo: u64,
    pub ids: u32,
}

trait IndexItem {
    fn src_id(&self) -> u32;
    fn name_id(&self) -> u32;
    fn packed_locinfo(&self, idx: usize) -> (u32, u32);

    fn dst_line(&self) -> u32 {
        self.packed_locinfo(0).0
    }

    fn dst_col(&self) -> u32 {
        self.packed_locinfo(0).1
    }

    fn src_line(&self) -> u32 {
        self.packed_locinfo(1).0
    }

    fn src_col(&self) -> u32 {
        self.packed_locinfo(1).1
    }
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

pub struct DumpOptions {
    pub with_source_contents: bool,
    pub with_names: bool,
}

enum Backing<'a> {
    Buf(Cow<'a, [u8]>),
    Mmap(Mmap),
}

pub struct MemDb<'a> {
    backing: Backing<'a>
}

pub struct Token<'a> {
    db: &'a MemDb<'a>,
    raw: RawToken,
}


fn verify_version<'a>(rv: MemDb<'a>) -> Result<MemDb<'a>> {
    if try!(rv.header()).version != 1 {
        Err(ErrorKind::UnsupportedMemDbVersion.into())
    } else {
        Ok(rv)
    }
}

fn pack_loc_shape(line: u32, col: u32) -> Result<(u8, u32)> {
    fn mask(x: u32, m: u32) -> Result<u32> {
        let p = x & m;
        if p != x {
            Err(ErrorKind::LocationOverflow.into())
        } else {
            Ok(p)
        }
    }

    Ok(if line > col {
        (1, (try!(mask(line, 0x1ffff)) << 14) | try!(mask(col, 0x3fff)))
    } else {
        (0, (try!(mask(line, 0x3fff)) << 17) | try!(mask(col, 0x1ffff)))
    })
}

fn unpack_loc_shape(shape: u8, packed: u32) -> (u32, u32) {
    if shape == 1 {
        (packed >> 14, packed & 0x3fff)
    } else {
        (packed >> 17, packed & 0x1ffff)
    }
}


impl LargeIndexItem {

    pub fn new(raw: &RawToken) -> Result<LargeIndexItem> {
        let psrc_id : u32 = raw.src_id & 0x3fff;
        let pname_id : u32 = raw.name_id & 0x3ffff;
        if raw.src_id != !0 && psrc_id >= 0x3fff {
            return Err(ErrorKind::TooManySources.into());
        }
        if raw.name_id != !0 && pname_id >= 0x3ffff {
            return Err(ErrorKind::TooManyNames.into());
        }

        let (sdst, pdst) = try!(pack_loc_shape(raw.dst_line, raw.dst_col));
        let (ssrc, psrc) = try!(pack_loc_shape(raw.src_line, raw.src_col));

        Ok(LargeIndexItem {
            packed_locinfo: (
                ((sdst as u64) << 63) |
                ((ssrc as u64) << 62) |
                ((pdst as u64) << 31) |
                (psrc as u64)
            ),
            ids: (psrc_id << 18) | pname_id,
        })
    }
}

impl IndexItem for LargeIndexItem {

    fn src_id(&self) -> u32 {
        let mut src_id : u32 = self.ids >> 18;
        if src_id == 0x3fff {
            src_id = !0;
        }
        src_id
    }

    fn name_id(&self) -> u32 {
        let mut name_id : u32 = self.ids & 0x3ffff;
        if name_id == 0x3ffff {
            name_id = !0;
        }
        name_id
    }

    fn packed_locinfo(&self, idx: usize) -> (u32, u32) {
        let (s1, s2) = if idx == 0 { (63, 31) } else { (62, 0) };
        unpack_loc_shape(((self.packed_locinfo >> s1) & 0x1) as u8,
                         ((self.packed_locinfo >> s2) & 0x7fffffff) as u32)
    }
}

impl<'a> MemDb<'a> {

    pub fn from_cow(cow: Cow<'a, [u8]>) -> Result<MemDb<'a>> {
        verify_version(MemDb {
            backing: Backing::Buf(cow),
        })
    }

    pub fn from_slice(buffer: &'a [u8]) -> Result<MemDb<'a>> {
        MemDb::from_cow(Cow::Borrowed(buffer))
    }

    pub fn from_vec(buffer: Vec<u8>) -> Result<MemDb<'a>> {
        MemDb::from_cow(Cow::Owned(buffer))
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<MemDb<'a>> {
        let mmap = try!(Mmap::open_path(path, Protection::Read));
        verify_version(MemDb {
            backing: Backing::Mmap(mmap),
        })
    }

    pub fn get_name(&self, name_id: u32) -> Option<&str> {
        self.names().ok().and_then(|x| self.get_string(x, name_id))
    }

    pub fn get_source(&self, src_id: u32) -> Option<&str> {
        self.sources().ok().and_then(|x| self.get_string(x, src_id))
    }

    pub fn get_source_contents(&'a self, src_id: u32) -> Option<String> {
        self.source_contents().ok().and_then(|x| {
            self.get_bytes(x, src_id)
        }).and_then(|bytes| {
            let mut decompr = BrotliDecoder::new(bytes);
            let mut contents = String::new();
            decompr.read_to_string(&mut contents).ok();
            Some(contents)
        })
    }

    pub fn get_token_count(&self) -> u32 {
        self.header().map(|x| x.index_size).unwrap_or(0)
    }

    pub fn get_source_count(&self) -> u32 {
        self.header().map(|x| x.sources_count).unwrap_or(0)
    }

    pub fn get_token(&'a self, idx: u32) -> Option<Token<'a>> {
        self.index_at(idx).map(|ii| {
            Token {
                db: self,
                raw: RawToken {
                    dst_line: ii.dst_line(),
                    dst_col: ii.dst_col(),
                    src_line: ii.src_line(),
                    src_col: ii.src_col(),
                    src_id: ii.src_id(),
                    name_id: ii.name_id(),
                }
            }
        })
    }

    pub fn lookup_token(&'a self, line: u32, col: u32) -> Option<Token<'a>> {
        let mut low = 0;
        let mut high = self.index_size();

        while low < high {
            let mid = (low + high) / 2;
            let ii = self.index_at(mid).unwrap();
            if (line, col) < (ii.dst_line(), ii.dst_col()) {
                high = mid;
            } else {
                low = mid + 1;
            }
        }

        if low > 0 && low <= self.index_size() {
            self.get_token(low as u32 - 1)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn buffer(&self) -> &[u8] {
        match self.backing {
            Backing::Buf(ref buf) => buf,
            Backing::Mmap(ref mmap) => unsafe { mmap.as_slice() }
        }
    }

    fn get_data(&self, start: usize, len: usize) -> Result<&[u8]> {
        let buffer = self.buffer();
        let end = start.wrapping_add(len);
        if end < start || end > buffer.len() {
            Err(ErrorKind::BadMemDb.into())
        } else {
            Ok(&buffer[start..end])
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

    fn get_bytes(&self, coll: &[u32], idx: u32) -> Option<&[u8]> {
        coll.get(idx as usize).and_then(|offset| {
            let mut offset = *offset as usize;
            let buffer = self.buffer();
            let mut len = 0u64;
            offset += varinteger::decode_with_offset(buffer, offset, &mut len) as usize;
            Some(&buffer[offset..offset + len as usize])
        })
    }

    fn get_string(&self, coll: &[u32], idx: u32) -> Option<&str> {
        self.get_bytes(coll, idx).and_then(|bytes| from_utf8(bytes).ok())
    }

    #[inline(always)]
    fn header(&self) -> Result<&MapHead> {
        unsafe {
            Ok(mem::transmute(try!(self.get_data(0, mem::size_of::<MapHead>())).as_ptr()))
        }
    }

    #[inline(always)]
    fn index_size(&self) -> u32 {
        self.header().map(|x| x.index_size).unwrap_or(0)
    }

    #[inline(always)]
    fn index_at(&self, idx: u32) -> Option<&IndexItem> {
        let index : Option<&[LargeIndexItem]> = self.header().ok().and_then(|head| {
            let off = mem::size_of::<MapHead>();
            self.get_slice(off, head.index_size as usize).ok()
        });
        index.and_then(|index| {
            index.get(idx as usize).map(|x| x as &IndexItem)
        })
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

fn write_str<W: Write>(w: &mut W, bytes: &[u8]) -> io::Result<u32> {
    let mut buf = [0u8; 8];
    let off = varinteger::encode(bytes.len() as u64, &mut buf);
    try!(w.write_all(&buf[..off]));
    try!(w.write_all(bytes));
    Ok(bytes.len() as u32 + off as u32)
}

fn write_slice<T, W: Write>(w: &mut W, x: &[T]) -> io::Result<u32> {
    unsafe {
        let bytes : *const u8 = mem::transmute(x.as_ptr());
        let size = mem::size_of::<T>() * x.len();
        try!(w.write_all(slice::from_raw_parts(bytes, size)));
        Ok(size as u32)
    }
}

fn sourcemap_to_memdb_common<W: Write>(sm: &SourceMap, mut w: W, opts: DumpOptions)
    -> Result<(W, MapHead)>
{
    let mut head = MapHead {
        version: 1,
        index_size: sm.get_index_size() as u32,
        names_start: 0,
        names_count: if opts.with_names { sm.get_name_count() } else { 0 },
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
        idx += try!(write_obj(&mut w, &try!(LargeIndexItem::new(&raw))));
    }

    // write names
    let names = {
        if opts.with_names {
            let mut names = Vec::with_capacity(sm.get_name_count() as usize);
            for name in sm.names() {
                names.push(idx);
                idx += try!(write_str(&mut w, name.as_bytes()));
            }
            names
        } else {
            vec![]
        }
    };

    // write sources
    let mut sources = Vec::with_capacity(sm.get_source_count() as usize);
    let mut source_contents = if opts.with_source_contents {
        Vec::with_capacity(sm.get_source_count() as usize)
    } else {
        vec![]
    };
    let mut have_sources = false;
    for source_id in 0..sm.get_source_count() {
        let source = sm.get_source(source_id).unwrap();
        sources.push(idx);
        idx += try!(write_str(&mut w, source.as_bytes()));

        if opts.with_source_contents {
            if let Some(contents) = sm.get_source_contents(source_id) {
                have_sources = true;
                source_contents.push(idx);
                let mut compressed = vec![];
                let mut compr = BrotliEncoder::new(contents.as_bytes(), 4);
                try!(compr.read_to_end(&mut compressed));
                idx += try!(write_str(&mut w, &compressed[..]));
            } else {
                source_contents.push(!0);
            }
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

    Ok((w, head))
}

/// Serializes a map into a vec
pub fn sourcemap_to_memdb_vec(sm: &SourceMap, opts: DumpOptions) -> Vec<u8> {
    let mut rv = vec![];
    let (_, head) = sourcemap_to_memdb_common(sm, &mut rv, opts).unwrap();

    unsafe {
        let byte_head : *const u8 = mem::transmute(&head);
        copy_nonoverlapping(byte_head, rv.as_mut_ptr(),
                            mem::size_of_val(&head));
    }

    rv
}

/// Serializes a map into a given writer
pub fn sourcemap_to_memdb<W: Write+Seek>(sm: &SourceMap, w: W, opts: DumpOptions)
    -> Result<()>
{
    let (mut w, head) = try!(sourcemap_to_memdb_common(sm, w, opts));

    // write offsets
    try!(w.seek(SeekFrom::Start(0)));
    try!(write_obj(&mut w, &head));

    Ok(())
}
