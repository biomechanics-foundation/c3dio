/// Bytes is a struct that contains the header, parameter, and data of a file.
/// The header is 512 bytes long and is always the first 512 bytes of the file.
/// The parameter data is variable length and is located after the header.
/// The data is variable length and is located after the parameter data.
/// The parameter data and data are stored in blocks of 512 bytes.
///
/// The `parameter_start_block_index` and `data_start_block_index` fields
/// are used to calculate the byte offset of the parameter and data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bytes {
    pub header: [u8; 512],
    pub parameter: Vec<u8>,
    pub data: Vec<u8>,
    pub parameter_start_block_index: usize,
    pub data_start_block_index: usize,
}

impl Default for Bytes {
    fn default() -> Self {
        Bytes {
            header: [0; 512],
            parameter: Vec::new(),
            data: Vec::new(),
            parameter_start_block_index: 0,
            data_start_block_index: 0,
        }
    }
}

impl Bytes {
    pub fn new() -> Bytes {
        Bytes::default()
    }
}
