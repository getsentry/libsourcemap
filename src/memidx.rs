use sourcemap::RawToken;

use errors::{ErrorKind, Result};


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


#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct LargeIndexItem {
    pub packed_locinfo: u64,
    pub ids: u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct CompactIndexItem {
    pub packed_locinfo: u64,
    pub src_id: u16,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct VeryCompactIndexItem {
    pub packed_info: u64,
}


pub trait IndexItem {
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

impl CompactIndexItem {

    pub fn new(raw: &RawToken) -> Result<CompactIndexItem> {
        let src_id = if raw.src_id >= 0xffff {
            return Err(ErrorKind::TooManySources.into());
        } else if raw.src_id == !0 {
            !0u16
        } else {
            raw.src_id as u16
        };

        let (sdst, pdst) = try!(pack_loc_shape(raw.dst_line, raw.dst_col));
        let (ssrc, psrc) = try!(pack_loc_shape(raw.src_line, raw.src_col));

        Ok(CompactIndexItem {
            packed_locinfo: (
                ((sdst as u64) << 63) |
                ((ssrc as u64) << 62) |
                ((pdst as u64) << 31) |
                (psrc as u64)
            ),
            src_id: src_id,
        })
    }
}

impl IndexItem for CompactIndexItem {

    fn src_id(&self) -> u32 {
        self.src_id as u32
    }

    fn name_id(&self) -> u32 {
        !0
    }

    fn packed_locinfo(&self, idx: usize) -> (u32, u32) {
        let (s1, s2) = if idx == 0 { (63, 31) } else { (62, 0) };
        unpack_loc_shape(((self.packed_locinfo >> s1) & 0x1) as u8,
                         ((self.packed_locinfo >> s2) & 0x7fffffff) as u32)
    }
}

impl VeryCompactIndexItem {

    pub fn new(raw: &RawToken) -> Result<VeryCompactIndexItem> {
        let src_id = if raw.src_id >= 0xffff {
            return Err(ErrorKind::TooManySources.into());
        } else if raw.src_id == !0 {
            !0u16
        } else {
            raw.src_id as u16
        };

        let (sdst, pdst) = try!(pack_loc_shape(raw.dst_line, raw.dst_col));
        if raw.src_line >= 0xffff || raw.src_col > 0 {
            return Err(ErrorKind::LocationOverflow.into());
        }

        Ok(VeryCompactIndexItem {
            packed_info: (
                ((sdst as u64) << 63) |
                ((pdst as u64) << 32) |
                ((raw.src_line as u64) << 16) |
                (src_id as u64)
            ),
        })
    }
}

impl IndexItem for VeryCompactIndexItem {

    fn src_id(&self) -> u32 {
        (self.packed_info & 0xffff) as u32
    }

    fn name_id(&self) -> u32 {
        !0
    }

    fn packed_locinfo(&self, idx: usize) -> (u32, u32) {
        if idx == 0 {
            unpack_loc_shape(((self.packed_info >> 63) & 0x1) as u8,
                             ((self.packed_info >> 32) & 0x7fffffff) as u32)
        } else {
            (((self.packed_info >> 16) & 0xffff) as u32, 0)
        }
    }
}
