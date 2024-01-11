use std::error::Error;
use std::io::{BufRead, Cursor, Read};
use std::simd::u8x16;

use super::instruction::Lane16;
use super::leb128;
use super::section::DataCountSeg;
use crate::execution::value::{v128, ToV128};

pub type DecodeResult<T> = Result<T, Box<dyn Error>>;

pub struct Reader<'a> {
    buf: Cursor<&'a [u8]>,
    pub data_count: DataCountSeg,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8], data_count: DataCountSeg) -> Self {
        let buf = Cursor::new(data);

        Self { buf, data_count }
    }

    pub fn bytes(&mut self, size: usize) -> DecodeResult<Vec<u8>> {
        let mut buf = vec![0u8; size];

        self.buf.read_exact(&mut buf)?;

        Ok(buf)
    }

    pub fn seqs(&mut self) -> DecodeResult<Vec<u8>> {
        let size = self.get_leb_u32()? as usize;

        self.bytes(size)
    }

    pub fn byte_16(&mut self) -> DecodeResult<Lane16> {
        let mut buf = [0u8; 16];

        self.buf.read_exact(&mut buf)?;

        Ok(buf)
    }

    pub fn not_end(&mut self) -> DecodeResult<bool> {
        Ok(self.buf.fill_buf().map(|b| !b.is_empty())?)
    }

    pub fn get_u8(&mut self) -> DecodeResult<u8> {
        let mut buf = [0u8; 1];

        self.buf.read_exact(&mut buf)?;

        Ok(buf[0])
    }

    #[inline]
    pub fn get_u32(&mut self) -> DecodeResult<u32> {
        let mut buf = [0u8; 4];

        self.buf.read_exact(&mut buf)?;

        Ok(u32::from_le_bytes(buf))
    }

    pub fn get_f32(&mut self) -> DecodeResult<f32> {
        let mut buf = [0u8; 4];

        self.buf.read_exact(&mut buf)?;

        Ok(f32::from_le_bytes(buf))
    }

    pub fn get_f64(&mut self) -> DecodeResult<f64> {
        let mut buf = [0u8; 8];

        self.buf.read_exact(&mut buf)?;

        Ok(f64::from_le_bytes(buf))
    }

    pub fn get_v128(&mut self) -> DecodeResult<v128> {
        let bytes = self.byte_16()?;
        let v = u8x16::from_array(bytes);

        Ok(v.v128())
    }

    pub fn get_leb_u32(&mut self) -> DecodeResult<u32> {
        let data = self.buf.remaining_slice();
        let (num, size) = leb128::decode_unsigned(data)?;

        self.buf.consume(size);

        Ok(num as u32)
    }

    pub fn get_leb_u64(&mut self) -> DecodeResult<u64> {
        let data = self.buf.remaining_slice();
        let (num, size) = leb128::decode_unsigned(data)?;

        self.buf.consume(size);

        Ok(num)
    }

    pub fn get_leb_i32(&mut self) -> DecodeResult<i32> {
        let data = self.buf.remaining_slice();
        let (num, size) = leb128::decode_signed(data, 32)?;

        self.buf.consume(size);

        Ok(num as i32)
    }

    pub fn get_leb_i64(&mut self) -> DecodeResult<i64> {
        let data = self.buf.remaining_slice();
        let (num, size) = leb128::decode_signed(data, 64)?;

        self.buf.consume(size);

        Ok(num)
    }

    pub fn get_name(&mut self) -> DecodeResult<String> {
        let bytes = self.seqs()?;
        let name = String::from_utf8(bytes)?;

        Ok(name)
    }

    #[inline]
    pub fn remain(&mut self) -> DecodeResult<Vec<u8>> {
        let len = self.buf.get_ref().len();
        let position = self.buf.position() as usize;

        self.bytes(len - position)
    }
}
