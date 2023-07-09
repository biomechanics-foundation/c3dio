
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bytes {
    pub header: [u8; 512],
    pub parameter: Vec<u8>,
    pub data: Vec<u8>,
    pub parameter_start_block_index: usize,
    pub data_start_block_index: usize,
}

impl Bytes {
    pub fn new() -> Bytes {
        Bytes {
            header: [0; 512],
            parameter: Vec::new(),
            data: Vec::new(),
            parameter_start_block_index: 0,
            data_start_block_index: 0,
        }
    }
}
