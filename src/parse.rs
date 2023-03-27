use std::fs;

use crate::processor::{ProcessorType, intel_u16, dec_u16, sgi_mips_u16};
use crate::header::{Header, parse_header};

#[derive(Debug)]
pub enum C3dParseError {
    ReadError(std::io::Error),
    InsufficientBlocks(String),
    InvalidHeaderStartBlock,
    InvalidParameterStartBlock,
    //    InvalidDataStartBlock,
    InvalidProcessorType,
}

pub fn read_c3d(file: &str) -> Result<Vec<u8>, C3dParseError> {
    let mut contents = match fs::read(file) {
        Ok(contents) => contents,
        Err(e) => return Err(C3dParseError::ReadError(e)),
    };
    if contents.len() % 512 != 0 {
        //        println!("Data is not a multiple of 512 bytes, adding padding");
        //        println!("Padding file with {} bytes", 512 - (contents.len() % 512));
        contents.resize(contents.len() + (512 - (contents.len() % 512)), 0);
    }

    Ok(contents)
}

pub fn parse_basic_info(contents: &Vec<u8>) -> Result<(u8, u16, ProcessorType), C3dParseError> {
    let parameter_start_block_index = *contents
        .get(0)
        .ok_or(C3dParseError::InvalidParameterStartBlock)?;
    let start_index = (parameter_start_block_index as u16 * 512) - 512 as u16;
    let end_index = start_index + 512;
    let parameter_start_block = contents
        .get(start_index as usize..end_index as usize)
        .ok_or(C3dParseError::InvalidParameterStartBlock)?;

    let endian = match get_endian(parameter_start_block.try_into().unwrap()) {
        Ok(endian) => endian,
        Err(e) => return Err(e),
    };

    let index_1 = contents
        .get(16)
        .ok_or(C3dParseError::InvalidProcessorType)?;
    let index_2 = contents
        .get(17)
        .ok_or(C3dParseError::InvalidProcessorType)?;

    let data_start_block = match endian {
        ProcessorType::Intel => intel_u16(&[*index_1, *index_2]),
        ProcessorType::Dec => dec_u16(&[*index_1, *index_2]),
        ProcessorType::SgiMips => sgi_mips_u16(&[*index_1, *index_2]),
    };

    Ok((parameter_start_block_index, data_start_block, endian))
}

pub fn split_c3d_contents_into_sections(
    contents: &Vec<u8>,
    parameter_start_block_index: u8,
    data_start_block_index: u16,
) -> Result<(Vec<&[u8; 512]>, Vec<&[u8; 512]>, Vec<&[u8; 512]>), C3dParseError> {
    let mut header_blocks: Vec<&[u8; 512]> = Vec::new();
    let mut parameter_blocks: Vec<&[u8; 512]> = Vec::new();
    let mut data_blocks: Vec<&[u8; 512]> = Vec::new();

    for i in 0..contents.len() / 512 as usize {
        let block: &[u8; 512] = match contents.get(i * 512..(i + 1) * 512) {
            Some(block) => <&[u8; 512]>::try_from(block).unwrap_or(&[0; 512]),
            None => {
                return Err(C3dParseError::InsufficientBlocks(format!(
                    "Block {i} is missing"
                )))
            }
        };
        if i < (parameter_start_block_index - 1) as usize {
            header_blocks.push(block);
        } else if i < (data_start_block_index - 1) as usize {
            parameter_blocks.push(block);
        } else {
            data_blocks.push(block);
        }
    }
    Ok((header_blocks, parameter_blocks, data_blocks))
}

pub fn parse_header_from_file(file: &str) -> Result<Header, C3dParseError> {
    let contents = read_c3d(file)?;

    let (parameter_start_block_index, data_start_block_index, endian) =
        parse_basic_info(&contents)?;

    let (header_blocks, _, _) = split_c3d_contents_into_sections(
        &contents,
        parameter_start_block_index,
        data_start_block_index,
    )?;

    let header = parse_header(
        header_blocks
            .get(0)
            .ok_or(C3dParseError::InvalidHeaderStartBlock)?,
        &endian,
    );

    Ok(header)
}

pub fn get_endian(first_parameter_block: &[u8; 512]) -> Result<ProcessorType, C3dParseError> {
    match first_parameter_block[3] {
        0x54 => Ok(ProcessorType::Intel),
        0x55 => Ok(ProcessorType::Dec),
        0x56 => Ok(ProcessorType::SgiMips),
        _ => Err(C3dParseError::InvalidProcessorType),
    }
}
