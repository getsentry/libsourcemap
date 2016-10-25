use byteorder::{BigEndian, ByteOrder};


pub fn clz(mut x: u32) -> u32 {
    let mut n = 32;
    let mut y;

    y = x >> 16;
    if y != 0 {
        n = n -16;
        x = y;
    }

    y = x >> 8;
    if y != 0 {
        n = n - 8;
        x = y;
    }

    y = x >> 4;
    if y != 0 {
        n = n - 4;
        x = y;
    }

    y = x >> 2;
    if y != 0 {
        n = n - 2;
        x = y;
    }

    y = x >> 1;

    if y != 0 {
        n - 2
    } else {
        n - x
    }
}

pub fn iclz(x: u32) -> u32 {
    32 - clz(x)
}


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
        if bits == 0 {
            return;
        }
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

    pub fn write_id(&mut self, mut value: u32, bits: usize) {
        if value == !0 {
            value = ((1 << bits) as u32) - 1;
        }
        self.write(value, bits)
    }

    pub fn flush(&mut self) {
        BigEndian::write_u32(&mut self.buf[self.offset..], self.bits_buf);
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
        if bits == 0 {
            return 0;
        }
        if bits > 32 {
            panic!("Bit overflow ({} > 32)", bits);
        };
        if self.buf.len() * 8 < self.bits + bits {
            panic!("Buffer not large enough");
        };
        let mut output = 0;
        loop {
            if self.bits_left == 0 {
                let mut backup_buf;
                let mut buf = &self.buf[self.offset..];

                // if we're not large enough for 4 byte, we need to pad us
                // out with some zeroes.
                if buf.len() < 4 {
                    backup_buf = [0u8; 4];
                    backup_buf[..buf.len()].copy_from_slice(&buf);
                    buf = &backup_buf;
                }

                self.bits_buf = BigEndian::read_u32(buf);
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

    pub fn read_id(&mut self, bits: usize) -> u32 {
        let rv = self.read(bits);
        if rv == ((1 << bits) as u32) - 1 {
            !0
        } else {
            rv
        }
    }
}
