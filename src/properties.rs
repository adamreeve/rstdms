use crate::error::{Result, TdmsReadError};
use num_traits::FromPrimitive;

use crate::types::{TdsType, TypeReader};

#[derive(Debug, PartialEq, Eq)]
pub enum TdmsValue {
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),
    String(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct TdmsProperty {
    pub name: String,
    pub value: TdmsValue,
}

fn read_value<T: TypeReader>(type_id: TdsType, reader: &mut T) -> Result<TdmsValue> {
    match type_id {
        TdsType::I8 => Ok(TdmsValue::Int8(reader.read_int8()?)),
        TdsType::I16 => Ok(TdmsValue::Int16(reader.read_int16()?)),
        TdsType::I32 => Ok(TdmsValue::Int32(reader.read_int32()?)),
        TdsType::I64 => Ok(TdmsValue::Int64(reader.read_int64()?)),
        TdsType::U8 => Ok(TdmsValue::Uint8(reader.read_uint8()?)),
        TdsType::U16 => Ok(TdmsValue::Uint16(reader.read_uint16()?)),
        TdsType::U32 => Ok(TdmsValue::Uint32(reader.read_uint32()?)),
        TdsType::U64 => Ok(TdmsValue::Uint64(reader.read_uint64()?)),
        TdsType::String => Ok(TdmsValue::String(reader.read_string()?)),
        _ => panic!("Unsupported type {:?}", type_id),
    }
}

impl TdmsProperty {
    pub fn read<T: TypeReader>(reader: &mut T) -> Result<TdmsProperty> {
        let name = reader.read_string()?;
        let type_id_raw = reader.read_uint32()?;
        let type_id = FromPrimitive::from_u32(type_id_raw);
        if let Some(type_id) = type_id {
            let value = read_value(type_id, reader)?;
            Ok(TdmsProperty { name, value })
        } else {
            Err(TdmsReadError::TdmsError(format!(
                "Invalid type id: {}",
                type_id_raw
            )))
        }
    }
}

#[cfg(test)]
mod test {
    extern crate hex_literal;

    use hex_literal::hex;
    use std::io::Cursor;

    use super::*;
    use crate::types::LittleEndianReader;

    #[test]
    pub fn can_read_int32_property() {
        let mut cursor = Cursor::new(hex!(
            "
            0D 00 00 00
            70 72 6F 70 65 72 74 79 20 6E 61 6D 65
            03 00 00 00
            0A 00 00 00
            "
        ));
        let mut reader = LittleEndianReader::new(&mut cursor);
        let property = TdmsProperty::read(&mut reader).unwrap();

        assert_eq!(property.name, "property name");
        assert_eq!(property.value, TdmsValue::Int32(10i32));
    }

    #[test]
    pub fn can_read_string_property() {
        let mut cursor = Cursor::new(hex!(
            "
            0D 00 00 00
            70 72 6F 70 65 72 74 79 20 6E 61 6D 65
            20 00 00 00
            0E 00 00 00
            70 72 6F 70 65 72 74 79 20 76 61 6C 75 65
            "
        ));
        let mut reader = LittleEndianReader::new(&mut cursor);
        let property = TdmsProperty::read(&mut reader).unwrap();

        assert_eq!(property.name, "property name");
        assert_eq!(
            property.value,
            TdmsValue::String(String::from("property value"))
        );
    }

    #[test]
    pub fn unexpected_end_of_data() {
        let mut cursor = Cursor::new(hex!(
            "
            0D 00 00 00
            70 72 6F 70 65 72
            "
        ));
        let mut reader = LittleEndianReader::new(&mut cursor);
        let error = TdmsProperty::read(&mut reader).unwrap_err();

        match error {
            TdmsReadError::IoError(_) => {}
            _ => panic!("Unexpected error variant"),
        }
    }

    #[test]
    pub fn invalid_utf8() {
        let mut cursor = Cursor::new(hex!(
            "
            0D 00 00 00
            FF FF FF FF FF FF FF FF FF FF FF FF FF
            "
        ));
        let mut reader = LittleEndianReader::new(&mut cursor);
        let error = TdmsProperty::read(&mut reader).unwrap_err();

        match error {
            TdmsReadError::Utf8Error(_) => {}
            _ => panic!("Unexpected error variant"),
        }
    }
}
