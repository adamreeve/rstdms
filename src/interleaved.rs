use std::cmp::min;
use std::io::{Read, Result};

/// Reads data for a single channel from a chunk of interleaved channel data
pub struct InterleavedReader<'a> {
    /// Chunk of bytes to read from
    bytes: &'a [u8],

    /// Overall width of all channels in this chunk
    chunk_width: usize,

    /// Width in bytes of the type of data in this channel
    type_size: usize,

    /// Offset within the data to the start of this channel
    offset: usize,

    /// Position in the output interleaved data
    position: usize,
}

impl<'a> InterleavedReader<'a> {
    pub fn new(
        bytes: &'a [u8],
        chunk_width: usize,
        type_size: usize,
        offset: usize,
    ) -> InterleavedReader<'a> {
        InterleavedReader {
            bytes,
            chunk_width,
            type_size,
            offset,
            position: 0,
        }
    }
}

impl<'a> Read for InterleavedReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let total_bytes = self.type_size * self.bytes.len() / self.chunk_width;
        let num_bytes_to_read = min(buf.len(), total_bytes - self.position);

        for i in 0..num_bytes_to_read {
            let position = i + self.position;
            let type_idx = position / self.type_size;
            let type_offset = position % self.type_size;
            buf[i] = self.bytes[self.offset + type_idx * self.chunk_width + type_offset];
        }

        self.position += num_bytes_to_read;
        Ok(num_bytes_to_read)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_empty_data() {
        let mut buffer = vec![0u8; 4];
        let bytes = vec![0u8; 0];
        let mut reader = InterleavedReader::new(&bytes, 8, 4, 0);

        let result = reader.read(&mut buffer);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn read_to_empty_buffer() {
        let mut buffer = vec![0u8; 0];
        let bytes = vec![0u8; 8];
        let mut reader = InterleavedReader::new(&bytes, 8, 4, 0);

        let result = reader.read(&mut buffer);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn read_to_required_size_buffer() {
        let mut buffer = vec![0u8; 8];
        let bytes = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let mut reader = InterleavedReader::new(&bytes, 4, 2, 0);

        let result = reader.read(&mut buffer);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 8);
        assert_eq!(buffer, vec![0, 1, 4, 5, 8, 9, 12, 13]);
    }

    #[test]
    fn read_to_larger_buffer() {
        let mut buffer = vec![0u8; 10];
        let bytes = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let mut reader = InterleavedReader::new(&bytes, 4, 2, 0);

        let result = reader.read(&mut buffer);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 8);
        assert_eq!(buffer, vec![0, 1, 4, 5, 8, 9, 12, 13, 0, 0]);
    }

    #[test]
    fn read_to_smaller_buffer() {
        let mut buffer = vec![0u8; 6];
        let bytes = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let mut reader = InterleavedReader::new(&bytes, 4, 2, 0);

        let result = reader.read(&mut buffer);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 6);
        assert_eq!(buffer, vec![0, 1, 4, 5, 8, 9]);
    }

    #[test]
    fn read_split() {
        let mut buffer = vec![0u8; 8];
        let bytes = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let mut reader = InterleavedReader::new(&bytes, 4, 2, 0);

        let result = reader.read(&mut buffer[0..6]);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 6);
        assert_eq!(buffer, vec![0, 1, 4, 5, 8, 9, 0, 0]);

        let result = reader.read(&mut buffer[6..8]);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(buffer, vec![0, 1, 4, 5, 8, 9, 12, 13]);
    }

    #[test]
    fn read_split_off_type_boundary() {
        let mut buffer = vec![0u8; 8];
        let bytes = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let mut reader = InterleavedReader::new(&bytes, 4, 2, 0);

        let result = reader.read(&mut buffer[0..5]);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5);
        assert_eq!(buffer, vec![0, 1, 4, 5, 8, 0, 0, 0]);

        let result = reader.read(&mut buffer[5..8]);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(buffer, vec![0, 1, 4, 5, 8, 9, 12, 13]);
    }

    #[test]
    fn read_split_off_type_boundary_with_offset() {
        let mut buffer = vec![0u8; 8];
        let bytes = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let mut reader = InterleavedReader::new(&bytes, 4, 2, 2);

        let result = reader.read(&mut buffer[0..5]);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5);
        assert_eq!(buffer, vec![2, 3, 6, 7, 10, 0, 0, 0]);

        let result = reader.read(&mut buffer[5..8]);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(buffer, vec![2, 3, 6, 7, 10, 11, 14, 15]);
    }
}
