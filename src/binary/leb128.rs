use super::reader::DecodeResult;
use crate::binary::errors::DecodeErr;

/// https://en.wikipedia.org/wiki/LEB128
pub fn decode_unsigned(data: &[u8]) -> DecodeResult<(u64, usize)> {
    let mut result = 0;

    for (i, b) in data.iter().enumerate() {
        result |= ((*b & 0b0111_1111) as u64) << (i * 7);

        // 最高位为 0，停止读入后续字节
        let no_more = b & 0b1000_0000 == 0;

        match i == 4 {
            true if !no_more => Err(DecodeErr::LEBDecodeTooLong)?,
            true if (b >> (32 - i * 7)) > 0 => Err(DecodeErr::IntTooLarge)?,
            _ if no_more => return Ok((result, i + 1)),
            _ => continue,
        }
    }

    Err(DecodeErr::LEBUnexpectedEnd)?
}

pub fn decode_signed(data: &[u8], size: usize) -> DecodeResult<(i64, usize)> {
    let mut result = 0;
    let mut shift = 0;

    for (i, b) in data.iter().enumerate() {
        result |= ((*b & 0b0111_1111) as i64) << shift;
        shift += 7;

        let no_more = b & 0b1000_0000 == 0;

        if i == size / 7 {
            match no_more {
                false => Err(DecodeErr::LEBDecodeTooLong)?,
                true => {
                    let o1 = b & 0x40 == 0 && b >> (size - i * 7 - 1) != 0;
                    let o2 = b & 0x40 != 0 && ((b | 0x80) as i8) >> (size - i * 7 - 1) != -1;

                    if o1 || o2 {
                        Err(DecodeErr::IntTooLarge)?
                    }
                }
            }
        }

        if no_more {
            // 第二高位为 1，表示复数，最高位全部补 1
            if shift < size && (b & 0b0100_0000 != 0) {
                result |= !0 << shift;
            }

            return Ok((result, i + 1));
        }
    }

    Err(DecodeErr::LEBUnexpectedEnd)?
}

pub fn encode_unsigned(mut data: u64) -> Vec<u8> {
    let mut result = vec![];

    loop {
        // 低 7 位
        let mut byte = (data as u8) & 0b0111_1111;
        // 将原值不断进行右移
        data >>= 7;

        // 存在后续字节，最高位设置为 1
        if data != 0 {
            byte |= 0b1000_0000;
        }

        result.push(byte);

        if data == 0 {
            break;
        }
    }

    result
}

pub fn encode_signed(mut data: i64) -> Vec<u8> {
    let mut result = vec![];
    let mut more = true;

    while more {
        let mut byte = (data as u8) & 0b0111_1111;

        data >>= 7;

        // 第二高位为符号位
        if (data == 0 && (byte & 0b0100_0000) == 0) || (data == -1 && (byte & 0b0100_0000) != 0) {
            more = false;
        } else {
            byte |= 0b1000_0000;
        }

        result.push(byte);
    }

    result
}

pub fn encode_u32(data: u32) -> Vec<u8> {
    encode_unsigned(data as u64)
}

pub fn encode_usize(data: usize) -> Vec<u8> {
    encode_unsigned(data as u64)
}

pub fn encode_name(name: &str) -> Vec<u8> {
    let bytes = name.as_bytes();

    [encode_usize(bytes.len()), bytes.to_vec()].concat()
}

#[cfg(test)]
mod test {
    use super::{decode_signed, decode_unsigned};
    use crate::binary::leb128::{encode_signed, encode_unsigned};

    #[test]
    fn test_decode_uint() {
        let data: &[u8] = &[
            0b1_0111111,
            0b1_0011111,
            0b1_0001111,
            0b1_0000111,
            0b1_0000011,
            0b0_0000001,
        ];
        let (num, size) = decode_unsigned(&data[5..]).unwrap();
        assert_eq!(num, 0b0000001);
        assert_eq!(size, 1);

        let (num, size) = decode_unsigned(&data[4..]).unwrap();
        assert_eq!(num, 0b1_0000011);
        assert_eq!(size, 2);

        let (num, size) = decode_unsigned(&data[3..]).unwrap();
        assert_eq!(num, 0b1_0000011_0000111);
        assert_eq!(size, 3);

        let (num, size) = decode_unsigned(&data[2..]).unwrap();
        assert_eq!(num, 0b1_0000011_0000111_0001111);
        assert_eq!(size, 4);

        let (num, size) = decode_unsigned(&data[1..]).unwrap();
        assert_eq!(num, 0b1_0000011_0000111_0001111_0011111);
        assert_eq!(size, 5);
    }

    #[test]
    fn test_decode_int() {
        let (num, size) = decode_signed(&[0xc0, 0xbb, 0x78], 32).unwrap();

        assert_eq!(num, -123456);
        assert_eq!(size, 3);
    }

    #[test]
    fn test_encode_unsigned() {
        let data: u64 = 624485;
        let e = encode_unsigned(data);
        let d = decode_unsigned(&e).unwrap();

        assert_eq!(data as u64, d.0);
    }

    #[test]
    fn test_encode_signed() {
        let datas: Vec<i64> = vec![624485, -123456, 1048576, 0];

        for data in datas {
            let e = encode_signed(data);
            let d = decode_signed(&e, 64).unwrap();

            assert_eq!(data, d.0);
        }
    }
}
