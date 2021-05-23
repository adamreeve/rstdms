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
    fn read_values<R: Read, O: ByteOrderExt>(
        target_buffer: &mut Vec<Self>,
        reader: &mut R,
        num_values: usize,
    ) -> Result<()>;
}

impl NativeType for i8 {
    fn native_type() -> NativeTypeId {
        NativeTypeId::I8
    }

    fn read_values<R: Read, O: ByteOrderExt>(
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

    fn read_values<R: Read, O: ByteOrderExt>(
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

    fn read_values<R: Read, O: ByteOrderExt>(
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

    fn read_values<R: Read, O: ByteOrderExt>(
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

    fn read_values<R: Read, O: ByteOrderExt>(
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

    fn read_values<R: Read, O: ByteOrderExt>(
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

    fn read_values<R: Read, O: ByteOrderExt>(
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

    fn read_values<R: Read, O: ByteOrderExt>(
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

    fn read_values<R: Read, O: ByteOrderExt>(
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

    fn read_values<R: Read, O: ByteOrderExt>(
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

    fn read_values<R: Read, O: ByteOrderExt>(
        target_buffer: &mut Vec<Self>,
        reader: &mut R,
        num_values: usize,
    ) -> Result<()> {
        let original_length = target_buffer.len();
        let new_length = original_length + num_values;
        target_buffer.resize(new_length, Timestamp::new(0, 0));
        for _ in 0..num_values {
            target_buffer.push(read_timestamp::<_, O>(reader)?);
        }
        Ok(())
    }
}

pub fn read_string<R: Read, O: ByteOrder>(reader: &mut R) -> Result<String> {
    let string_length = reader.read_u32::<O>()?;

    let mut string_bytes = vec![0; string_length as usize];
    reader.read_exact(&mut string_bytes)?;
    Ok(String::from_utf8(string_bytes)?)
}

pub fn read_timestamp<R: Read, O: ByteOrderExt>(reader: &mut R) -> std::io::Result<Timestamp> {
    let mut buf = [0; 16];
    reader.read_exact(&mut buf)?;
    Ok(O::read_timestamp(&buf))
}

pub trait ByteOrderExt: ByteOrder {
    fn read_timestamp(buf: &[u8]) -> Timestamp;
}

impl ByteOrderExt for LittleEndian {
    fn read_timestamp(buf: &[u8]) -> Timestamp {
        let second_fractions = Self::read_u64(&buf[0..8]);
        let seconds = Self::read_i64(&buf[8..16]);
        Timestamp::new(seconds, second_fractions)
    }
}

impl ByteOrderExt for BigEndian {
    fn read_timestamp(buf: &[u8]) -> Timestamp {
        let seconds = Self::read_i64(&buf[0..8]);
        let second_fractions = Self::read_u64(&buf[8..16]);
        Timestamp::new(seconds, second_fractions)
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
    pub fn can_read_string_le() {
        let mut reader = Cursor::new(hex!("05 00 00 00 68 65 6C 6C 6F"));
        let value = read_string::<_, LittleEndian>(&mut reader).unwrap();

        assert_eq!(value, "hello");
    }

    #[test]
    pub fn can_read_string_be() {
        let mut reader = Cursor::new(hex!("00 00 00 05 68 65 6C 6C 6F"));
        let value = read_string::<_, BigEndian>(&mut reader).unwrap();

        assert_eq!(value, "hello");
    }
}
