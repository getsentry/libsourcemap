use byteorder::{LittleEndian, ByteOrder};


#[derive(Debug, PartialEq, Eq)]
pub struct BitPacker<'a> {
    pub buf: &'a mut [u8],
    pub offset: usize,
    pub bits_left: usize,
    pub bits_buf: u32,
    pub bits: usize

}

#[derive(Debug, PartialEq, Eq)]
pub struct BitUnpacker<'a> {
    pub buf: &'a [u8],
    pub offset: usize,
    pub bits_left: usize,
    pub bits_buf: u32,
    pub bits: usize
}

impl<'a> BitPacker<'a> {

    pub fn new(buf: &'a mut [u8]) -> BitPacker<'a> {
        BitPacker {
            buf: buf,
            bits_left: 32,
            bits_buf: 0,
            bits: 0,
            offset: 0
        }
    }

    pub fn write(&mut self, mut value: u32, mut bits: usize) {
        if bits > 32 {
            panic!("Bit overflow ({} > 32)", bits);
        };
        if bits < 32 {
            value &= (1 << bits) - 1;
        }
        if self.buf.len() * 8 < self.bits + bits {
            panic!("Buffer not large enough");
        };
        self.bits += bits;

        loop {
            if bits <= self.bits_left {
                self.bits_buf |= value << (self.bits_left - bits);
                self.bits_left -= bits;
                break;
            }

            self.bits_buf |= value >> (bits - self.bits_left);
            value &= (1 << (bits - self.bits_left)) - 1;
            bits -= self.bits_left;

            self.flush();
        }
    }

    pub fn flush(&mut self) {
        if self.bits_buf == 0 {
            return;
        }
        LittleEndian::write_u32(&mut self.buf[self.offset..], self.bits_buf);
        self.offset += 4;
        self.bits_buf = 0;
        self.bits_left = 32;
    }
}

impl<'a> BitUnpacker<'a> {

    pub fn new(buf: &'a [u8]) -> BitUnpacker<'a> {
        BitUnpacker {
            buf: buf,
            bits_left: 0,
            bits_buf: 0,
            bits: 0,
            offset: 0
        }
    }

    pub fn read(&mut self, mut bits: usize) -> u32 {
        if bits > 32 {
            panic!("Bit overflow ({} > 32)", bits);
        };
        if self.buf.len() * 8 < self.bits + bits {
            panic!("Buffer not large enough");
        };
        let mut output = 0;
        loop {
            if self.bits_left == 0 {
                self.bits_buf = LittleEndian::read_u32(&self.buf[self.offset..]);
                self.offset += 4;
                self.bits_left = 32;
            }
            if bits <= self.bits_left {
                output |= self.bits_buf >> (self.bits_left - bits);
                self.bits_buf &= (1 << (self.bits_left - bits)) - 1;
                self.bits_left -= bits;
                break;
            }
            output |= self.bits_buf << (bits - self.bits_left);
            bits -= self.bits_left;
            self.bits_left = 0;
        }

        output
    }
}
