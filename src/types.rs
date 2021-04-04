use std::io::Read;
use crate::error::Result;

#[derive(FromPrimitive, Debug)]
#[repr(u32)]
pub enum TdsType {
    Void = 0,
    I8 = 1,
    I16 = 2,
    I32 = 3,
    I64 = 4,
    U8 = 5,
    U16 = 6,
    U32 = 7,
    U64 = 8,
    SingleFloat = 9,
    DoubleFloat = 10,
    ExtendedFloat = 11,
    SingleFloatWithUnit = 0x19,
    DoubleFloatWithUnit = 0x1A,
    ExtendedFloatWithUnit = 0x1B,
    String = 0x20,
    Boolean = 0x21,
    TimeStamp = 0x44,
    FixedPoint = 0x4F,
    ComplexSingleFloat = 0x08000C,
    ComplexDoubleFloat = 0x10000D,
    DaqmxRawData = 0xFFFFFFFF,
}

pub trait TypeReader {
    fn read_int8(&mut self) -> Result<i8>;
    fn read_int16(&mut self) -> Result<i16>;
    fn read_int32(&mut self) -> Result<i32>;
    fn read_int64(&mut self) -> Result<i64>;
    fn read_uint8(&mut self) -> Result<u8>;
    fn read_uint16(&mut self) -> Result<u16>;
    fn read_uint32(&mut self) -> Result<u32>;
    fn read_uint64(&mut self) -> Result<u64>;
    fn read_float32(&mut self) -> Result<f32>;
    fn read_float64(&mut self) -> Result<f64>;
    fn read_string(&mut self) -> Result<String>;
}

pub struct LittleEndianReader<T: Read> {
    reader: T,
}

impl<T: Read> LittleEndianReader<T> {
    pub fn new(reader: T) -> LittleEndianReader<T> {
        LittleEndianReader { reader }
    }

    pub fn into_inner(self) -> T {
        self.reader
    }
}

impl<T: Read> TypeReader for LittleEndianReader<T> {
    fn read_int8(&mut self) -> Result<i8> {
        let mut buffer = [0; 1];
        self.reader.read_exact(&mut buffer)?;
        Ok(i8::from_le_bytes(buffer))
    }

    fn read_int16(&mut self) -> Result<i16> {
        let mut buffer = [0; 2];
        self.reader.read_exact(&mut buffer)?;
        Ok(i16::from_le_bytes(buffer))
    }

    fn read_int32(&mut self) -> Result<i32> {
        let mut buffer = [0; 4];
        self.reader.read_exact(&mut buffer)?;
        Ok(i32::from_le_bytes(buffer))
    }

    fn read_int64(&mut self) -> Result<i64> {
        let mut buffer = [0; 8];
        self.reader.read_exact(&mut buffer)?;
        Ok(i64::from_le_bytes(buffer))
    }

    fn read_uint8(&mut self) -> Result<u8> {
        let mut buffer = [0; 1];
        self.reader.read_exact(&mut buffer)?;
        Ok(u8::from_le_bytes(buffer))
    }

    fn read_uint16(&mut self) -> Result<u16> {
        let mut buffer = [0; 2];
        self.reader.read_exact(&mut buffer)?;
        Ok(u16::from_le_bytes(buffer))
    }

    fn read_uint32(&mut self) -> Result<u32> {
        let mut buffer = [0; 4];
        self.reader.read_exact(&mut buffer)?;
        Ok(u32::from_le_bytes(buffer))
    }

    fn read_uint64(&mut self) -> Result<u64> {
        let mut buffer = [0; 8];
        self.reader.read_exact(&mut buffer)?;
        Ok(u64::from_le_bytes(buffer))
    }

    fn read_float32(&mut self) -> Result<f32> {
        let mut buffer = [0; 4];
        self.reader.read_exact(&mut buffer)?;
        Ok(f32::from_le_bytes(buffer))
    }

    fn read_float64(&mut self) -> Result<f64> {
        let mut buffer = [0; 8];
        self.reader.read_exact(&mut buffer)?;
        Ok(f64::from_le_bytes(buffer))
    }

    fn read_string(&mut self) -> Result<String> {
        let mut length_buffer = [0; 4];
        self.reader.read_exact(&mut length_buffer)?;
        let string_length = u32::from_le_bytes(length_buffer);

        let mut string_bytes = vec![0; string_length as usize];
        self.reader.read_exact(&mut string_bytes)?;
        Ok(String::from_utf8(string_bytes)?)
    }
}

#[cfg(test)]
mod test {
    extern crate hex_literal;

    use hex_literal::hex;
    use std::io::Cursor;

    use super::*;

    #[test]
    pub fn can_read_int8_le() {
        let cursor = Cursor::new(hex!("FE"));
        let mut reader = LittleEndianReader::new(cursor);
        let value = reader.read_int8().unwrap();

        assert_eq!(value, -2i8);
    }

    #[test]
    pub fn can_read_int16_le() {
        let cursor = Cursor::new(hex!("FE FF"));
        let mut reader = LittleEndianReader::new(cursor);
        let value = reader.read_int16().unwrap();

        assert_eq!(value, -2i16);
    }

    #[test]
    pub fn can_read_int32_le() {
        let cursor = Cursor::new(hex!("FE FF FF FF"));
        let mut reader = LittleEndianReader::new(cursor);
        let value = reader.read_int32().unwrap();

        assert_eq!(value, -2i32);
    }

    #[test]
    pub fn can_read_int64_le() {
        let cursor = Cursor::new(hex!("FE FF FF FF FF FF FF FF"));
        let mut reader = LittleEndianReader::new(cursor);
        let value = reader.read_int64().unwrap();

        assert_eq!(value, -2i64);
    }

    #[test]
    pub fn can_read_uint8_le() {
        let cursor = Cursor::new(hex!("FE"));
        let mut reader = LittleEndianReader::new(cursor);
        let value = reader.read_uint8().unwrap();

        assert_eq!(value, 254u8);
    }

    #[test]
    pub fn can_read_uint16_le() {
        let cursor = Cursor::new(hex!("FE FF"));
        let mut reader = LittleEndianReader::new(cursor);
        let value = reader.read_uint16().unwrap();

        assert_eq!(value, 65534u16);
    }

    #[test]
    pub fn can_read_uint32_le() {
        let cursor = Cursor::new(hex!("FE FF FF FF"));
        let mut reader = LittleEndianReader::new(cursor);
        let value = reader.read_uint32().unwrap();

        assert_eq!(value, 4294967294u32);
    }

    #[test]
    pub fn can_read_uint64_le() {
        let cursor = Cursor::new(hex!("FE FF FF FF FF FF FF FF"));
        let mut reader = LittleEndianReader::new(cursor);
        let value = reader.read_uint64().unwrap();

        assert_eq!(value, 18446744073709551614u64);
    }

    #[test]
    pub fn can_read_float32_le() {
        let cursor = Cursor::new(hex!("A4 70 45 41"));
        let mut reader = LittleEndianReader::new(cursor);
        let value = reader.read_float32().unwrap();

        assert_eq!(value, 12.34f32);
    }

    #[test]
    pub fn can_read_float64_le() {
        let cursor = Cursor::new(hex!("AE 47 E1 7A 14 AE 28 40"));
        let mut reader = LittleEndianReader::new(cursor);
        let value = reader.read_float64().unwrap();

        assert_eq!(value, 12.34f64);
    }

    #[test]
    pub fn can_read_string_le() {
        let cursor = Cursor::new(hex!("05 00 00 00 68 65 6C 6C 6F"));
        let mut reader = LittleEndianReader::new(cursor);
        let value = reader.read_string().unwrap();

        assert_eq!(value, "hello");
    }
}
