use crate::data::{parse_data, Data};
use crate::header::{parse_header, Header};
use crate::parameters::{parse_parameters, Parameters};
use crate::parse::{parse_basic_info, read_c3d, split_c3d, C3dParseError};

#[derive(Debug)]
pub struct C3d {
    pub file_name: String,
    pub header: Header,
    pub parameters: Parameters,
    pub data: Data,
}

impl C3d {
    pub fn from_file(file_name: &str) -> Result<C3d, C3dParseError> {
        let contents = read_c3d(file_name)?;

        let (parameter_start_block_index, data_start_block, processor_type) =
            parse_basic_info(contents.as_slice())?;

        let (header_blocks, parameter_blocks, data_blocks) =
            split_c3d(contents.as_slice(), parameter_start_block_index, data_start_block)?;

        let header = parse_header(header_blocks, &processor_type)?;
        let parameters = parse_parameters(parameter_blocks, &processor_type)?;
        let data = parse_data(data_blocks, &parameters, &processor_type)?;

        Ok(C3d {
            file_name: file_name.to_string(),
            header,
            parameters,
            data,
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<C3d, C3dParseError> {
        let (parameter_start_block_index, data_start_block, processor_type) =
            parse_basic_info(bytes)?;

        let (header_blocks, parameter_blocks, data_blocks) =
            split_c3d(bytes, parameter_start_block_index, data_start_block)?;

        let header = parse_header(header_blocks, &processor_type)?;
        let parameters = parse_parameters(parameter_blocks, &processor_type)?;
        let data = parse_data(data_blocks, &parameters, &processor_type)?;

        Ok(C3d {
            file_name: "bytes".to_string(),
            header,
            parameters,
            data,
        })
    }
}
