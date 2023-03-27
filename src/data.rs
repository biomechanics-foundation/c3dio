use crate::processor::ProcessorType;

pub struct Data {}

pub fn parse_data(data_blocks: &Vec<&[u8; 512]>, processor_type: &ProcessorType) -> Data {
    data_blocks;
    processor_type;
    Data {}
}
