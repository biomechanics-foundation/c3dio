use std::fs::File;
use std::path::PathBuf;

#[derive(Debug)]
pub enum C3dParseError {
    ReadError(std::io::Error),
    InsufficientBlocks(String),
    InvalidHeaderStartBlock,
    InvalidParameterStartBlock,
    InvalidParameterData,
    InvalidDataStartBlock,
    InvalidProcessorType,
    InvalidDataType,
    InvalidParametersOffset,
    MissingGroup(String),
    MissingParameter(String),
    UnknownDataFormat,
    InvalidGroupId,
    MissingPointScale,
}

pub fn open_c3d(file: &PathBuf) -> Result<File, C3dParseError> {
    let file = File::open(file).map_err(|e| C3dParseError::ReadError(e))?;

    Ok(file)
}

pub fn split_c3d(
    contents: &[u8],
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
