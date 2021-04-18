extern crate hex_literal;

use hex_literal::hex;
use std::io::Cursor;

use rstdms::TdmsFile;
use typed_arena::Arena;

struct TestFile {
    bytes: Vec<u8>,
}

impl TestFile {
    fn new() -> TestFile {
        TestFile { bytes: Vec::new() }
    }

    fn add_segment(&mut self, metadata_bytes: &Vec<u8>, data_bytes: &Vec<u8>) {
        // TDSm tag
        self.bytes.extend(&hex!("54 44 53 6D"));

        // ToC mask
        let toc_mask: u32 = (1 << 1) | (1 << 2) | (1 << 3);
        self.bytes.extend(&toc_mask.to_le_bytes());

        // Version number
        self.bytes.extend(&hex!("69 12 00 00"));

        // Offsets
        let raw_data_offset = metadata_bytes.len();
        let next_segment_offset = raw_data_offset + data_bytes.len();
        self.bytes
            .extend(&(next_segment_offset as u64).to_le_bytes());
        self.bytes.extend(&(raw_data_offset as u64).to_le_bytes());

        self.bytes.extend(metadata_bytes);
        self.bytes.extend(data_bytes);
    }

    fn to_cursor(self) -> Cursor<Vec<u8>> {
        Cursor::new(self.bytes)
    }
}

fn write_string(string: &str, bytes: &mut Vec<u8>) {
    bytes.extend(&(string.len() as u32).to_le_bytes());
    bytes.extend(string.bytes());
}

#[test]
fn read_metadata() {
    let mut metadata_bytes = Vec::new();

    // Number of objects
    metadata_bytes.extend(&(2_u32.to_le_bytes()));

    // Object path
    write_string("/", &mut metadata_bytes);
    // Raw data index
    metadata_bytes.extend(&hex!("FF FF FF FF"));
    // Number of properties
    metadata_bytes.extend(&(1_u32.to_le_bytes()));
    // Property
    write_string("test_property", &mut metadata_bytes);
    metadata_bytes.extend(&(3_u32.to_le_bytes()));
    metadata_bytes.extend(&(10_i32.to_le_bytes()));

    // Object path
    write_string("/'Group'/'Channel1'", &mut metadata_bytes);
    // Raw data index
    metadata_bytes.extend(&(20_u32.to_le_bytes())); // Raw data index length
    metadata_bytes.extend(&(3_u32.to_le_bytes())); // Data type
    metadata_bytes.extend(&(1_u32.to_le_bytes())); // Dimension
    metadata_bytes.extend(&(3_u64.to_le_bytes())); // Number of values

    // Number of properties
    metadata_bytes.extend(&(0_u32.to_le_bytes()));

    let mut data_bytes = Vec::new();
    data_bytes.extend(&(1_i32.to_le_bytes()));
    data_bytes.extend(&(2_i32.to_le_bytes()));
    data_bytes.extend(&(3_i32.to_le_bytes()));

    let mut test_file = TestFile::new();
    test_file.add_segment(&metadata_bytes, &data_bytes);

    let arena = Arena::new();
    let tdms_file = TdmsFile::new(test_file.to_cursor(), &arena);

    assert!(
        tdms_file.is_ok(),
        format!("Got error: {:?}", tdms_file.unwrap_err())
    );
}
