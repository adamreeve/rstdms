use crate::error::{Result, TdmsReadError};
use crate::timestamp::Timestamp;
use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::io::Read;

#[derive(Clone, Copy, TryFromPrimitive, Debug, PartialEq, Eq)]
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

pub trait ByteOrderWrapper: ByteOrder {
    fn is_little_endian() -> bool;
}

impl ByteOrderWrapper for LittleEndian {
    fn is_little_endian() -> bool {
        true
    }
}

impl ByteOrderWrapper for BigEndian {
    fn is_little_endian() -> bool {
        false
    }
}

impl TdsType {
    pub fn from_u32(type_id_raw: u32) -> Result<TdsType> {
        TdsType::try_from(type_id_raw)
            .map_err(|_| TdmsReadError::TdmsError(format!("Invalid type id: {}", type_id_raw)))
    }

    pub fn size(&self) -> Option<u32> {
        match *self {
            TdsType::Void => Some(0),
            TdsType::I8 => Some(1),
            TdsType::I16 => Some(2),
            TdsType::I32 => Some(4),
            TdsType::I64 => Some(8),
            TdsType::U8 => Some(1),
            TdsType::U16 => Some(2),
            TdsType::U32 => Some(4),
            TdsType::U64 => Some(8),
            TdsType::SingleFloat => Some(4),
            TdsType::DoubleFloat => Some(8),
            TdsType::ExtendedFloat => Some(16),
            TdsType::SingleFloatWithUnit => Some(4),
            TdsType::DoubleFloatWithUnit => Some(8),
            TdsType::ExtendedFloatWithUnit => Some(16),
            TdsType::String => None,
            TdsType::Boolean => Some(1),
            TdsType::TimeStamp => Some(16),
            TdsType::FixedPoint => None,
            TdsType::ComplexSingleFloat => Some(8),
            TdsType::ComplexDoubleFloat => Some(16),
            TdsType::DaqmxRawData => None,
        }
    }

    pub fn native_type(&self) -> Option<NativeTypeId> {
        match *self {
            TdsType::Void => None,
            TdsType::I8 => Some(NativeTypeId::I8),
            TdsType::I16 => Some(NativeTypeId::I16),
            TdsType::I32 => Some(NativeTypeId::I32),
            TdsType::I64 => Some(NativeTypeId::I64),
            TdsType::U8 => Some(NativeTypeId::U8),
            TdsType::U16 => Some(NativeTypeId::U16),
            TdsType::U32 => Some(NativeTypeId::U32),
            TdsType::U64 => Some(NativeTypeId::U64),
            TdsType::SingleFloat => Some(NativeTypeId::F32),
            TdsType::DoubleFloat => Some(NativeTypeId::F64),
            TdsType::ExtendedFloat => None,
            TdsType::SingleFloatWithUnit => Some(NativeTypeId::F32),
            TdsType::DoubleFloatWithUnit => Some(NativeTypeId::F64),
            TdsType::ExtendedFloatWithUnit => None,
            TdsType::String => None,
            TdsType::Boolean => None,
            TdsType::TimeStamp => Some(NativeTypeId::Timestamp),
            TdsType::FixedPoint => None,
            TdsType::ComplexSingleFloat => None,
            TdsType::ComplexDoubleFloat => None,
            TdsType::DaqmxRawData => None,
        }
    }
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
    fn read_timestamp(&mut self) -> Result<Timestamp>;
}

pub struct LittleEndianReader<'a, R: Read> {
    reader: &'a mut R,
}

impl<'a, R: Read> LittleEndianReader<'a, R> {
    pub fn new(reader: &'a mut R) -> LittleEndianReader<'a, R> {
        LittleEndianReader { reader }
    }
}

impl<'a, R: Read> TypeReader for LittleEndianReader<'a, R> {
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

    fn read_timestamp(&mut self) -> Result<Timestamp> {
        let mut buffer = [0; 8];
        self.reader.read_exact(&mut buffer)?;
        let second_fractions = u64::from_le_bytes(buffer);
        let mut buffer = [0; 8];
        self.reader.read_exact(&mut buffer)?;
        let seconds = i64::from_le_bytes(buffer);
        Ok(Timestamp::new(seconds, second_fractions))
    }
}

/// Represents a native rust type that TDMS channel data can be read as.
#[derive(Debug, PartialEq)]
pub enum NativeTypeId {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Timestamp,
}

/// A native rust type that TDMS channel data can be read as.
/// This is a sealed trait that cannot be implemented outside this crate.
pub trait NativeType: private::SealedNativeType + Sized {
    #[doc(hidden)]
    fn native_type() -> NativeTypeId;

    #[doc(hidden)]
    fn read_values<R: Read, O: ByteOrderWrapper>(
        target_buffer: &mut Vec<Self>,
        reader: &mut R,
        num_values: usize,
    ) -> Result<()>;
}

impl NativeType for i8 {
    fn native_type() -> NativeTypeId {
        NativeTypeId::I8
    }

    fn read_values<R: Read, O: ByteOrderWrapper>(
        target_buffer: &mut Vec<Self>,
        reader: &mut R,
        num_values: usize,
    ) -> Result<()> {
        let original_length = target_buffer.len();
        let new_length = original_length + num_values;
        target_buffer.resize(new_length, 0);
        reader.read_i8_into(&mut target_buffer[original_length..new_length])?;
        Ok(())
    }
}

impl NativeType for i16 {
    fn native_type() -> NativeTypeId {
        NativeTypeId::I16
    }

    fn read_values<R: Read, O: ByteOrderWrapper>(
        target_buffer: &mut Vec<Self>,
        reader: &mut R,
        num_values: usize,
    ) -> Result<()> {
        let original_length = target_buffer.len();
        let new_length = original_length + num_values;
        target_buffer.resize(new_length, 0);
        reader.read_i16_into::<O>(&mut target_buffer[original_length..new_length])?;
        Ok(())
    }
}

impl NativeType for i32 {
    fn native_type() -> NativeTypeId {
        NativeTypeId::I32
    }

    fn read_values<R: Read, O: ByteOrderWrapper>(
        target_buffer: &mut Vec<Self>,
        reader: &mut R,
        num_values: usize,
    ) -> Result<()> {
        let original_length = target_buffer.len();
        let new_length = original_length + num_values;
        target_buffer.resize(new_length, 0);
        reader.read_i32_into::<O>(&mut target_buffer[original_length..new_length])?;
        Ok(())
    }
}

impl NativeType for i64 {
    fn native_type() -> NativeTypeId {
        NativeTypeId::I64
    }

    fn read_values<R: Read, O: ByteOrderWrapper>(
        target_buffer: &mut Vec<Self>,
        reader: &mut R,
        num_values: usize,
    ) -> Result<()> {
        let original_length = target_buffer.len();
        let new_length = original_length + num_values;
        target_buffer.resize(new_length, 0);
        reader.read_i64_into::<O>(&mut target_buffer[original_length..new_length])?;
        Ok(())
    }
}

impl NativeType for u8 {
    fn native_type() -> NativeTypeId {
        NativeTypeId::U8
    }

    fn read_values<R: Read, O: ByteOrderWrapper>(
        target_buffer: &mut Vec<Self>,
        reader: &mut R,
        num_values: usize,
    ) -> Result<()> {
        let original_length = target_buffer.len();
        let new_length = original_length + num_values;
        target_buffer.resize(new_length, 0);
        reader.read_exact(&mut target_buffer[original_length..new_length])?;
        Ok(())
    }
}

impl NativeType for u16 {
    fn native_type() -> NativeTypeId {
        NativeTypeId::U16
    }

    fn read_values<R: Read, O: ByteOrderWrapper>(
        target_buffer: &mut Vec<Self>,
        reader: &mut R,
        num_values: usize,
    ) -> Result<()> {
        let original_length = target_buffer.len();
        let new_length = original_length + num_values;
        target_buffer.resize(new_length, 0);
        reader.read_u16_into::<O>(&mut target_buffer[original_length..new_length])?;
        Ok(())
    }
}

impl NativeType for u32 {
    fn native_type() -> NativeTypeId {
        NativeTypeId::U32
    }

    fn read_values<R: Read, O: ByteOrderWrapper>(
        target_buffer: &mut Vec<Self>,
        reader: &mut R,
        num_values: usize,
    ) -> Result<()> {
        let original_length = target_buffer.len();
        let new_length = original_length + num_values;
        target_buffer.resize(new_length, 0);
        reader.read_u32_into::<O>(&mut target_buffer[original_length..new_length])?;
        Ok(())
    }
}

impl NativeType for u64 {
    fn native_type() -> NativeTypeId {
        NativeTypeId::U64
    }

    fn read_values<R: Read, O: ByteOrderWrapper>(
        target_buffer: &mut Vec<Self>,
        reader: &mut R,
        num_values: usize,
    ) -> Result<()> {
        let original_length = target_buffer.len();
        let new_length = original_length + num_values;
        target_buffer.resize(new_length, 0);
        reader.read_u64_into::<O>(&mut target_buffer[original_length..new_length])?;
        Ok(())
    }
}

impl NativeType for f32 {
    fn native_type() -> NativeTypeId {
        NativeTypeId::F32
    }

    fn read_values<R: Read, O: ByteOrderWrapper>(
        target_buffer: &mut Vec<Self>,
        reader: &mut R,
        num_values: usize,
    ) -> Result<()> {
        let original_length = target_buffer.len();
        let new_length = original_length + num_values;
        target_buffer.resize(new_length, 0.0);
        reader.read_f32_into::<O>(&mut target_buffer[original_length..new_length])?;
        Ok(())
    }
}

impl NativeType for f64 {
    fn native_type() -> NativeTypeId {
        NativeTypeId::F64
    }

    fn read_values<R: Read, O: ByteOrderWrapper>(
        target_buffer: &mut Vec<Self>,
        reader: &mut R,
        num_values: usize,
    ) -> Result<()> {
        let original_length = target_buffer.len();
        let new_length = original_length + num_values;
        target_buffer.resize(new_length, 0.0);
        reader.read_f64_into::<O>(&mut target_buffer[original_length..new_length])?;
        Ok(())
    }
}

impl NativeType for Timestamp {
    fn native_type() -> NativeTypeId {
        NativeTypeId::Timestamp
    }

    fn read_values<R: Read, O: ByteOrderWrapper>(
        target_buffer: &mut Vec<Self>,
        reader: &mut R,
        num_values: usize,
    ) -> Result<()> {
        let original_length = target_buffer.len();
        let new_length = original_length + num_values;
        target_buffer.resize(new_length, Timestamp::new(0, 0));
        for _ in 0..num_values {
            if O::is_little_endian() {
                let second_fractions = reader.read_u64::<O>()?;
                let seconds = reader.read_i64::<O>()?;
                target_buffer.push(Timestamp::new(seconds, second_fractions));
            } else {
                let seconds = reader.read_i64::<O>()?;
                let second_fractions = reader.read_u64::<O>()?;
                target_buffer.push(Timestamp::new(seconds, second_fractions));
            }
        }
        Ok(())
    }
}

mod private {
    use crate::timestamp::Timestamp;

    pub trait SealedNativeType {}

    impl SealedNativeType for i8 {}
    impl SealedNativeType for i16 {}
    impl SealedNativeType for i32 {}
    impl SealedNativeType for i64 {}
    impl SealedNativeType for u8 {}
    impl SealedNativeType for u16 {}
    impl SealedNativeType for u32 {}
    impl SealedNativeType for u64 {}
    impl SealedNativeType for f32 {}
    impl SealedNativeType for f64 {}
    impl SealedNativeType for Timestamp {}
}

#[cfg(test)]
mod test {
    extern crate hex_literal;

    use hex_literal::hex;
    use std::io::Cursor;

    use super::*;

    #[test]
    pub fn can_read_int8_le() {
        let mut cursor = Cursor::new(hex!("FE"));
        let mut reader = LittleEndianReader::new(&mut cursor);
        let value = reader.read_int8().unwrap();

        assert_eq!(value, -2i8);
    }

    #[test]
    pub fn can_read_int16_le() {
        let mut cursor = Cursor::new(hex!("FE FF"));
        let mut reader = LittleEndianReader::new(&mut cursor);
        let value = reader.read_int16().unwrap();

        assert_eq!(value, -2i16);
    }

    #[test]
    pub fn can_read_int32_le() {
        let mut cursor = Cursor::new(hex!("FE FF FF FF"));
        let mut reader = LittleEndianReader::new(&mut cursor);
        let value = reader.read_int32().unwrap();

        assert_eq!(value, -2i32);
    }

    #[test]
    pub fn can_read_int64_le() {
        let mut cursor = Cursor::new(hex!("FE FF FF FF FF FF FF FF"));
        let mut reader = LittleEndianReader::new(&mut cursor);
        let value = reader.read_int64().unwrap();

        assert_eq!(value, -2i64);
    }

    #[test]
    pub fn can_read_uint8_le() {
        let mut cursor = Cursor::new(hex!("FE"));
        let mut reader = LittleEndianReader::new(&mut cursor);
        let value = reader.read_uint8().unwrap();

        assert_eq!(value, 254u8);
    }

    #[test]
    pub fn can_read_uint16_le() {
        let mut cursor = Cursor::new(hex!("FE FF"));
        let mut reader = LittleEndianReader::new(&mut cursor);
        let value = reader.read_uint16().unwrap();

        assert_eq!(value, 65534u16);
    }

    #[test]
    pub fn can_read_uint32_le() {
        let mut cursor = Cursor::new(hex!("FE FF FF FF"));
        let mut reader = LittleEndianReader::new(&mut cursor);
        let value = reader.read_uint32().unwrap();

        assert_eq!(value, 4294967294u32);
    }

    #[test]
    pub fn can_read_uint64_le() {
        let mut cursor = Cursor::new(hex!("FE FF FF FF FF FF FF FF"));
        let mut reader = LittleEndianReader::new(&mut cursor);
        let value = reader.read_uint64().unwrap();

        assert_eq!(value, 18446744073709551614u64);
    }

    #[test]
    pub fn can_read_float32_le() {
        let mut cursor = Cursor::new(hex!("A4 70 45 41"));
        let mut reader = LittleEndianReader::new(&mut cursor);
        let value = reader.read_float32().unwrap();

        assert_eq!(value, 12.34f32);
    }

    #[test]
    pub fn can_read_float64_le() {
        let mut cursor = Cursor::new(hex!("AE 47 E1 7A 14 AE 28 40"));
        let mut reader = LittleEndianReader::new(&mut cursor);
        let value = reader.read_float64().unwrap();

        assert_eq!(value, 12.34f64);
    }

    #[test]
    pub fn can_read_string_le() {
        let mut cursor = Cursor::new(hex!("05 00 00 00 68 65 6C 6C 6F"));
        let mut reader = LittleEndianReader::new(&mut cursor);
        let value = reader.read_string().unwrap();

        assert_eq!(value, "hello");
    }
}
