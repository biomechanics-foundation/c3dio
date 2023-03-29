use std::fs;

use crate::header::{parse_header, Header};
use crate::processor::{bytes_to_u16, get_processor, ProcessorType};

#[derive(Debug)]
pub enum C3dParseError {
    ReadError(std::io::Error),
    InsufficientBlocks(String),
    InvalidHeaderStartBlock,
    InvalidParameterStartBlock,
    InvalidDataStartBlock,
    InvalidProcessorType,
    InvalidDataType,
    InvalidParametersOffset,
}

pub fn read_c3d(file: &str) -> Result<Vec<u8>, C3dParseError> {
    let contents = match fs::read(file) {
        Ok(contents) => contents,
        Err(e) => return Err(C3dParseError::ReadError(e)),
    };

    Ok(contents)
}

pub fn parse_basic_info(contents: &Vec<u8>) -> Result<(u8, u16, ProcessorType), C3dParseError> {
    if contents.len() < 512 {
        return Err(C3dParseError::InsufficientBlocks(
            "Header block is missing".to_string(),
        ));
    }

    let parameter_start_block_index = contents[0];
    let start_index = (parameter_start_block_index - 1) as usize * 512;
    let end_index = start_index + 512;

    if end_index > contents.len() as usize {
        return Err(C3dParseError::InvalidParameterStartBlock);
    }

    let parameter_start_block = contents.split_at(start_index as usize).1;

    let processor_type = match get_processor(parameter_start_block) {
        Ok(processor_type) => processor_type,
        Err(e) => return Err(e),
    };

    let index_1 = contents
        .get(16)
        .ok_or(C3dParseError::InvalidProcessorType)?;
    let index_2 = contents
        .get(17)
        .ok_or(C3dParseError::InvalidProcessorType)?;

    let data_start_block = bytes_to_u16(&[*index_1, *index_2], &processor_type);

    Ok((
        parameter_start_block_index,
        data_start_block,
        processor_type,
    ))
}

pub fn split_c3d(
    contents: &Vec<u8>,
    parameter_start_block_index: u8,
    data_start_block_index: u16,
) -> Result<(&[u8], &[u8], &[u8]), C3dParseError> {
    if contents.len() < 512 {
        return Err(C3dParseError::InvalidHeaderStartBlock);
    }

    if (parameter_start_block_index - 1) as usize * 512 > contents.len() as usize {
        return Err(C3dParseError::InvalidParameterStartBlock);
    }

    if (data_start_block_index - 1) * 512 > contents.len() as u16 {
        return Err(C3dParseError::InvalidDataStartBlock);
    }

    let (header_blocks, parameters_and_data_blocks) =
        contents.split_at((parameter_start_block_index - 1) as usize * 512);
    let (parameter_blocks, data_blocks) = parameters_and_data_blocks
        .split_at((data_start_block_index - parameter_start_block_index as u16) as usize * 512);

    Ok((header_blocks, parameter_blocks, data_blocks))
}
