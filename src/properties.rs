use crate::error::{Result, TdmsReadError};
use crate::timestamp::Timestamp;
use byteorder::ReadBytesExt;
use std::io::Read;

use crate::types::{read_string, read_timestamp, ByteOrderExt, TdsType};

#[derive(Debug, PartialEq)]
pub enum TdmsValue {
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),
    Float32(f32),
    Float64(f64),
    String(String),
    Timestamp(Timestamp),
}

#[derive(Debug, PartialEq)]
pub struct TdmsProperty {
    pub name: String,
    pub value: TdmsValue,
}

fn read_value<R: Read, O: ByteOrderExt>(type_id: TdsType, reader: &mut R) -> Result<TdmsValue> {
    match type_id {
        TdsType::I8 => Ok(TdmsValue::Int8(reader.read_i8()?)),
        TdsType::I16 => Ok(TdmsValue::Int16(reader.read_i16::<O>()?)),
        TdsType::I32 => Ok(TdmsValue::Int32(reader.read_i32::<O>()?)),
        TdsType::I64 => Ok(TdmsValue::Int64(reader.read_i64::<O>()?)),
        TdsType::U8 => Ok(TdmsValue::Uint8(reader.read_u8()?)),
        TdsType::U16 => Ok(TdmsValue::Uint16(reader.read_u16::<O>()?)),
        TdsType::U32 => Ok(TdmsValue::Uint32(reader.read_u32::<O>()?)),
        TdsType::U64 => Ok(TdmsValue::Uint64(reader.read_u64::<O>()?)),
        TdsType::SingleFloat => Ok(TdmsValue::Float32(reader.read_f32::<O>()?)),
        TdsType::DoubleFloat => Ok(TdmsValue::Float64(reader.read_f64::<O>()?)),
        TdsType::String => Ok(TdmsValue::String(read_string::<R, O>(reader)?)),
        TdsType::TimeStamp => Ok(TdmsValue::Timestamp(read_timestamp::<R, O>(reader)?)),
        _ => Err(TdmsReadError::TdmsError(format!(
            "Unsupported property type {:?}",
            type_id
        ))),
    }
}

impl TdmsProperty {
    pub fn read<R: Read, O: ByteOrderExt>(reader: &mut R) -> Result<TdmsProperty> {
        let name = read_string::<R, O>(reader)?;
        let type_id_raw = reader.read_u32::<O>()?;
        let type_id = TdsType::from_u32(type_id_raw)?;
        let value = read_value::<R, O>(type_id, reader)?;
        Ok(TdmsProperty { name, value })
    }
}

#[cfg(test)]
mod test {
    extern crate hex_literal;

    use byteorder::LittleEndian;
    use chrono::{Duration, TimeZone, Utc};
    use hex_literal::hex;
    use std::io::Cursor;

    use super::*;
    use crate::error::TdmsReadError;

    #[test]
    pub fn can_read_int32_property() {
        let mut reader = Cursor::new(hex!(
            "
            0D 00 00 00
            70 72 6F 70 65 72 74 79 20 6E 61 6D 65
            03 00 00 00
            0A 00 00 00
            "
        ));
        let property = TdmsProperty::read::<_, LittleEndian>(&mut reader).unwrap();

        assert_eq!(property.name, "property name");
        assert_eq!(property.value, TdmsValue::Int32(10i32));
    }

    #[test]
    pub fn can_read_string_property() {
        let mut reader = Cursor::new(hex!(
            "
            0D 00 00 00
            70 72 6F 70 65 72 74 79 20 6E 61 6D 65
            20 00 00 00
            0E 00 00 00
            70 72 6F 70 65 72 74 79 20 76 61 6C 75 65
            "
        ));
        let property = TdmsProperty::read::<_, LittleEndian>(&mut reader).unwrap();

        assert_eq!(property.name, "property name");
        assert_eq!(
            property.value,
            TdmsValue::String(String::from("property value"))
        );
    }

    #[test]
    pub fn can_read_timestamp_property() {
        let mut reader = Cursor::new(hex!(
            "
            0D 00 00 00
            70 72 6F 70 65 72 74 79 20 6E 61 6D 65
            44 00 00 00
            00 08 89 A1 8C A9 54 AB
            7B 63 14 D2 00 00 00 00
            "
        ));
        let property = TdmsProperty::read::<_, LittleEndian>(&mut reader).unwrap();

        assert_eq!(property.name, "property name");
        assert_eq!(
            property.value,
            TdmsValue::Timestamp(Timestamp::new(3524551547, 1234567890 * 10u64.pow(10)))
        );

        if let TdmsValue::Timestamp(ts) = property.value {
            let expected_time = Utc
                .ymd(2015, 9, 8)
                .and_hms(10, 5, 47)
                .checked_add_signed(Duration::nanoseconds(669260594))
                .unwrap();
            assert_eq!(ts.to_datetime(), Some(expected_time));
        }
    }

    #[test]
    pub fn unexpected_end_of_data() {
        let mut reader = Cursor::new(hex!(
            "
            0D 00 00 00
            70 72 6F 70 65 72
            "
        ));
        let error = TdmsProperty::read::<_, LittleEndian>(&mut reader).unwrap_err();

        match error {
            TdmsReadError::IoError(_) => {}
            _ => panic!("Unexpected error variant"),
        }
    }

    #[test]
    pub fn invalid_utf8() {
        let mut reader = Cursor::new(hex!(
            "
            0D 00 00 00
            FF FF FF FF FF FF FF FF FF FF FF FF FF
            "
        ));
        let error = TdmsProperty::read::<_, LittleEndian>(&mut reader).unwrap_err();

        match error {
            TdmsReadError::Utf8Error(_) => {}
            _ => panic!("Unexpected error variant"),
        }
    }
}
