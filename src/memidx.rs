use std::cmp;
use std::io::Write;

use sourcemap::RawToken;

use errors::Result;
use bitpacker::{BitPacker, BitUnpacker, iclz};


fn idsz(id: u32) -> u32 {
    if id == !0 {
        1
    } else {
        iclz(id) + 1
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct IndexLayout {
    pub dst_line_bits: u8,
    pub dst_col_bits: u8,
    pub src_line_bits: u8,
    pub src_col_bits: u8,
    pub src_id_bits: u8,
    pub name_id_bits: u8,
}

impl IndexLayout {
    pub fn new() -> IndexLayout {
        IndexLayout {
            dst_line_bits: 0,
            dst_col_bits: 0,
            src_line_bits: 0,
            src_col_bits: 0,
            src_id_bits: 0,
            name_id_bits: 0,
        }
    }

    pub fn reshape(&mut self, raw: &RawToken, with_names: bool) {
        self.dst_line_bits = cmp::max(self.dst_line_bits, iclz(raw.dst_line) as u8);
        self.dst_col_bits = cmp::max(self.dst_col_bits, iclz(raw.dst_col) as u8);
        self.src_line_bits = cmp::max(self.src_line_bits, iclz(raw.src_line) as u8);
        self.src_col_bits = cmp::max(self.src_col_bits, iclz(raw.src_col) as u8);
        self.src_id_bits = cmp::max(self.src_id_bits, idsz(raw.src_id) as u8);
        if with_names {
            self.name_id_bits = cmp::max(self.name_id_bits, idsz(raw.name_id) as u8);
        }
    }

    pub fn item_size(&self) -> u32 {
        (((
            self.dst_line_bits +
            self.dst_col_bits + 
            self.src_line_bits +
            self.src_col_bits + 
            self.src_id_bits +
            self.name_id_bits
        ) as f32) / 8.0).ceil() as u32
    }

    pub fn write_token<W: Write>(&self, w: &mut W, raw: &RawToken) -> Result<u32> {
        let mut buf = [0u8; 64];
        {
            let mut bp = BitPacker::new(&mut buf);
            bp.write(raw.dst_line, self.dst_line_bits as usize);
            bp.write(raw.dst_col, self.dst_col_bits as usize);
            bp.write(raw.src_line, self.src_line_bits as usize);
            bp.write(raw.src_col, self.src_col_bits as usize);
            bp.write_id(raw.src_id, self.src_id_bits as usize);
            bp.write_id(raw.name_id, self.name_id_bits as usize);
            bp.flush();
        }
        let sz = self.item_size();
        try!(w.write_all(&buf[..sz as usize]));
        Ok(sz)
    }

    pub fn load_token(&self, bytes: &[u8]) -> RawToken {
        let mut bup = BitUnpacker::new(bytes);
        RawToken {
            dst_line: bup.read(self.dst_line_bits as usize),
            dst_col: bup.read(self.dst_col_bits as usize),
            src_line: bup.read(self.src_line_bits as usize),
            src_col: bup.read(self.src_col_bits as usize),
            src_id: bup.read_id(self.src_id_bits as usize),
            name_id: bup.read_id(self.name_id_bits as usize),
        }
    }
}

#[test]
fn test_token_writing() {
    let tok = RawToken {
        dst_line: 42,
        dst_col: 23,
        src_line: 1025,
        src_col: 0,
        src_id: 11,
        name_id: 23,
    };

    let mut layout = IndexLayout::new();
    layout.reshape(&tok, true);
    assert_eq!(layout.item_size(), 4);
    let mut buf = vec![];
    assert_eq!(layout.write_token(&mut buf, &tok).unwrap(), 4);
    let tok2 = layout.load_token(&buf);
    assert_eq!(tok, tok2);

    let large_tok = RawToken {
        dst_line: 421234,
        dst_col: 1024,
        src_line: 1025,
        src_col: 0,
        src_id: 11,
        name_id: 23,
    };

    let mut layout = IndexLayout::new();
    layout.reshape(&large_tok, true);
    assert_eq!(layout.item_size(), 7);
    let mut buf = vec![];
    assert_eq!(layout.write_token(&mut buf, &large_tok).unwrap(), 7);
    let large_tok2 = layout.load_token(&buf);
    assert_eq!(large_tok, large_tok2);

    let vlarge_tok = RawToken {
        dst_line: 421234,
        dst_col: 1024,
        src_line: 1025,
        src_col: 3,
        src_id: 11,
        name_id: 23232,
    };

    let mut layout = IndexLayout::new();
    layout.reshape(&vlarge_tok, true);
    assert_eq!(layout.item_size(), 8);
    let mut buf = vec![];
    assert_eq!(layout.write_token(&mut buf, &vlarge_tok).unwrap(), 8);
    let vlarge_tok2 = layout.load_token(&buf);
    assert_eq!(vlarge_tok, vlarge_tok2);
}
