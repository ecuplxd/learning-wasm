use std::error::Error;
use std::io::{BufRead, Cursor, Read};
use std::simd::u8x16;

use super::instruction::Lane16;
use super::leb128;
use crate::execution::types::{v128, ToV128};

pub type ReadResult<T> = Result<T, Box<dyn Error>>;

pub struct Reader<'a> {
    buf: Cursor<&'a [u8]>,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let buf = Cursor::new(data);

        Self { buf }
    }

    pub fn bytes(&mut self, size: usize) -> ReadResult<Vec<u8>> {
        let mut buf = vec![0u8; size];

        self.buf.read_exact(&mut buf)?;

        Ok(buf)
    }

    pub fn seqs(&mut self) -> ReadResult<Vec<u8>> {
        let size = self.get_leb_u32()? as usize;

        self.bytes(size)
    }

    pub fn byte_16(&mut self) -> ReadResult<Lane16> {
        let mut buf = [0u8; 16];

        self.buf.read_exact(&mut buf)?;

        Ok(buf)
    }

    pub fn not_end(&mut self) -> ReadResult<bool> {
        Ok(self.buf.fill_buf().map(|b| !b.is_empty())?)
    }

    pub fn get_u8(&mut self) -> ReadResult<u8> {
        let mut buf = [0u8; 1];

        self.buf.read_exact(&mut buf)?;

        Ok(buf[0])
    }

    #[inline]
    pub fn get_u32(&mut self) -> ReadResult<u32> {
        let mut buf = [0u8; 4];

        self.buf.read_exact(&mut buf)?;

        Ok(u32::from_le_bytes(buf))
    }

    pub fn get_f32(&mut self) -> ReadResult<f32> {
        let mut buf = [0u8; 4];

        self.buf.read_exact(&mut buf)?;

        Ok(f32::from_le_bytes(buf))
    }

    pub fn get_f64(&mut self) -> ReadResult<f64> {
        let mut buf = [0u8; 8];

        self.buf.read_exact(&mut buf)?;

        Ok(f64::from_le_bytes(buf))
    }

    pub fn get_v128(&mut self) -> ReadResult<v128> {
        let bytes = self.byte_16()?;
        let v = u8x16::from_array(bytes);

        Ok(v.v128())
    }

    pub fn get_leb_u32(&mut self) -> ReadResult<u32> {
        let data = self.buf.remaining_slice();
        let (num, size) = leb128::decode_unsigned(data);

        self.buf.consume(size);

        Ok(num as u32)
    }

    pub fn get_leb_u64(&mut self) -> ReadResult<u64> {
        let data = self.buf.remaining_slice();
        let (num, size) = leb128::decode_unsigned(data);

        self.buf.consume(size);

        Ok(num)
    }

    pub fn get_leb_i32(&mut self) -> ReadResult<i32> {
        let data = self.buf.remaining_slice();
        let (num, size) = leb128::decode_signed(data, 32);

        self.buf.consume(size);

        Ok(num as i32)
    }

    pub fn get_leb_i64(&mut self) -> ReadResult<i64> {
        let data = self.buf.remaining_slice();
        let (num, size) = leb128::decode_signed(data, 64);

        self.buf.consume(size);

        Ok(num)
    }

    pub fn get_name(&mut self) -> ReadResult<String> {
        let bytes = self.seqs()?;
        let name = String::from_utf8(bytes)?;

        Ok(name)
    }

    #[inline]
    pub fn remain(&mut self) -> ReadResult<Vec<u8>> {
        let len = self.buf.get_ref().len();
        let position = self.buf.position() as usize;

        self.bytes(len - position)
    }
}
