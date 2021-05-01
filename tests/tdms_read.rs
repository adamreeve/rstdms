extern crate hex_literal;

use hex_literal::hex;
use std::io::Cursor;

use rstdms::TdmsFile;

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

fn object_metadata(
    path: &'static str,
    raw_data_index: &[u8],
    properties: Vec<(&'static str, u32, &[u8])>,
) -> Vec<u8> {
    let mut metadata_bytes = Vec::new();
    write_string(path, &mut metadata_bytes);
    metadata_bytes.extend(raw_data_index);
    metadata_bytes.extend(&((properties.len() as u32).to_le_bytes()));
    for (name, type_id, val) in properties {
        write_string(name, &mut metadata_bytes);
        metadata_bytes.extend(&(type_id.to_le_bytes()));
        metadata_bytes.extend(val);
    }
    metadata_bytes
}

fn raw_data_index(data_type: u32, number_of_values: u64) -> Vec<u8> {
    let mut index_bytes = Vec::new();
    index_bytes.extend(&(20_u32.to_le_bytes())); // Raw data index length
    index_bytes.extend(&(data_type.to_le_bytes())); // Data type
    index_bytes.extend(&(1_u32.to_le_bytes())); // Dimension
    index_bytes.extend(&(number_of_values.to_le_bytes())); // Number of values
    index_bytes
}

fn metadata(objects: Vec<Vec<u8>>) -> Vec<u8> {
    let mut metadata_bytes = Vec::new();
    metadata_bytes.extend(&((objects.len() as u32).to_le_bytes()));
    for object in objects {
        metadata_bytes.extend(object);
    }
    metadata_bytes
}

fn data_bytes_i32(data: Vec<i32>) -> Vec<u8> {
    let mut bytes = Vec::new();
    for val in data {
        bytes.extend(&(val.to_le_bytes()));
    }
    bytes
}

fn write_string(string: &str, bytes: &mut Vec<u8>) {
    bytes.extend(&(string.len() as u32).to_le_bytes());
    bytes.extend(string.bytes());
}

#[test]
fn read_metadata() {
    let mut test_file = TestFile::new();
    let metadata_bytes = metadata(vec![
        object_metadata(
            "/",
            &hex!("FF FF FF FF"),
            vec![("test_property", 3, &10_i32.to_le_bytes())],
        ),
        object_metadata("/'Group'/'Channel1'", &raw_data_index(3, 3), Vec::new()),
    ]);
    let data_bytes = data_bytes_i32(vec![1, 2, 3]);
    test_file.add_segment(&metadata_bytes, &data_bytes);

    let tdms_file = TdmsFile::new(test_file.to_cursor());

    assert!(
        tdms_file.is_ok(),
        format!("Got error: {:?}", tdms_file.unwrap_err())
    );

    let mut tdms_file = tdms_file.unwrap();
    let mut group = tdms_file.group("Group").unwrap();
    let mut channel = group.channel("Channel1").unwrap();
    let mut data: Vec<i32> = Vec::new();
    channel.read_data(&mut data).unwrap();

    assert_eq!(data, vec![1, 2, 3]);
}

#[test]
fn read_metadata_with_repeated_raw_data_index() {
    let mut test_file = TestFile::new();
    let metadata_bytes = metadata(vec![object_metadata(
        "/'Group'/'Channel1'",
        &raw_data_index(3, 3),
        Vec::new(),
    )]);
    let data_bytes = data_bytes_i32(vec![1, 2, 3]);
    test_file.add_segment(&metadata_bytes, &data_bytes);
    let metadata_bytes = metadata(vec![object_metadata(
        "/'Group'/'Channel1'",
        &(0_u32.to_le_bytes()), // Raw data index matches previous
        Vec::new(),
    )]);
    test_file.add_segment(&metadata_bytes, &data_bytes);

    let tdms_file = TdmsFile::new(test_file.to_cursor());

    assert!(
        tdms_file.is_ok(),
        format!("Got error: {:?}", tdms_file.unwrap_err())
    );

    let mut tdms_file = tdms_file.unwrap();
    let mut group = tdms_file.group("Group").unwrap();
    let mut channel = group.channel("Channel1").unwrap();
    let mut data: Vec<i32> = Vec::new();
    channel.read_data(&mut data).unwrap();

    assert_eq!(data, vec![1, 2, 3, 1, 2, 3]);
}

#[test]
fn multiple_channels() {
    let mut test_file = TestFile::new();
    let metadata_bytes = metadata(vec![
        object_metadata("/'Group'/'Channel1'", &raw_data_index(3, 2), Vec::new()),
        object_metadata("/'Group'/'Channel2'", &raw_data_index(3, 3), Vec::new()),
        object_metadata("/'Group'/'Channel3'", &raw_data_index(3, 4), Vec::new()),
    ]);
    let data_bytes = data_bytes_i32(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    test_file.add_segment(&metadata_bytes, &data_bytes);

    let tdms_file = TdmsFile::new(test_file.to_cursor());

    assert!(
        tdms_file.is_ok(),
        format!("Got error: {:?}", tdms_file.unwrap_err())
    );

    let mut tdms_file = tdms_file.unwrap();
    let mut group = tdms_file.group("Group").unwrap();

    let expected_data = vec![
        vec![1, 2],
        vec![3, 4, 5],
        vec![6, 7, 8, 9],
    ];

    for (i, channel_name) in vec!["Channel1", "Channel2", "Channel3"].iter().enumerate() {
        let mut channel = group.channel(channel_name).unwrap();
        let mut data: Vec<i32> = Vec::new();
        channel.read_data(&mut data).unwrap();
        assert_eq!(data, expected_data[i]);
    }
}
