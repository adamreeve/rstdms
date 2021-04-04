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

fn read_value<T: TypeReader>(type_id: TdsType, reader: &mut T) -> TdmsValue {
    match type_id {
        TdsType::I8 => TdmsValue::Int8(reader.read_int8()),
        TdsType::I16 => TdmsValue::Int16(reader.read_int16()),
        TdsType::I32 => TdmsValue::Int32(reader.read_int32()),
        TdsType::I64 => TdmsValue::Int64(reader.read_int64()),
        TdsType::U8 => TdmsValue::Uint8(reader.read_uint8()),
        TdsType::U16 => TdmsValue::Uint16(reader.read_uint16()),
        TdsType::U32 => TdmsValue::Uint32(reader.read_uint32()),
        TdsType::U64 => TdmsValue::Uint64(reader.read_uint64()),
        TdsType::String => TdmsValue::String(reader.read_string()),
        _ => panic!("Unsupported type {:?}", type_id),
    }
}

impl TdmsProperty {
    pub fn read<T: TypeReader>(reader: &mut T) -> TdmsProperty {
        let name = reader.read_string();
        let type_id = FromPrimitive::from_u32(reader.read_uint32()).unwrap();
        let value = read_value(type_id, reader);
        TdmsProperty { name, value }
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
        let cursor = Cursor::new(hex!(
            "
            0D 00 00 00
            70 72 6F 70 65 72 74 79 20 6E 61 6D 65
            03 00 00 00
            0A 00 00 00
            "
        ));
        let mut reader = LittleEndianReader::new(cursor);
        let property = TdmsProperty::read(&mut reader);

        assert_eq!(property.name, "property name");
        assert_eq!(property.value, TdmsValue::Int32(10i32));
    }

    #[test]
    pub fn can_read_string_property() {
        let cursor = Cursor::new(hex!(
            "
            0D 00 00 00
            70 72 6F 70 65 72 74 79 20 6E 61 6D 65
            20 00 00 00
            0E 00 00 00
            70 72 6F 70 65 72 74 79 20 76 61 6C 75 65
            "
        ));
        let mut reader = LittleEndianReader::new(cursor);
        let property = TdmsProperty::read(&mut reader);

        assert_eq!(property.name, "property name");
        assert_eq!(
            property.value,
            TdmsValue::String(String::from("property value"))
        );
    }
}
