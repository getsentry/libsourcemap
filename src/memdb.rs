use std::str::from_utf8;
use std::mem;
use std::fmt;
use std::slice;

use sourcemap::RawToken;

use types::{MapHead, IndexItem, StringMarker};

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
        self.db.sources.get(self.raw.src_id as usize).and_then(|marker| {
            let bytes = &self.db.buffer[
                marker.pos as usize..(marker.pos + marker.len) as usize];
            from_utf8(bytes).ok()
        }).unwrap_or("")
    }

    /// get the name if it exists as string
    pub fn get_name(&self) -> Option<&str> {
        self.db.names.get(self.raw.name_id as usize).and_then(|marker| {
            let bytes = &self.db.buffer[
                marker.pos as usize..(marker.pos + marker.len) as usize];
            from_utf8(bytes).ok()
        })
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
