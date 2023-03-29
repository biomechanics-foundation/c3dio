use crate::parse::{parse_basic_info, read_c3d, split_c3d, C3dParseError};
use crate::processor::{bytes_to_f32, bytes_to_i16, bytes_to_u16, get_processor, ProcessorType};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DataType {
    Char = -1,
    Byte = 1,
    Integer = 2,
    Float = 4,
}

impl TryFrom<i8> for DataType {
    type Error = C3dParseError;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            -1 => Ok(DataType::Char),
            1 => Ok(DataType::Byte),
            2 => Ok(DataType::Integer),
            4 => Ok(DataType::Float),
            _ => Err(C3dParseError::InvalidDataType),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ParameterData {
    Char(Vec<char>),
    Byte(Vec<u8>),
    Integer(Vec<i16>),
    Float(Vec<f32>),
}

#[derive(Debug, Clone)]
pub struct Parameters {
    groups: Vec<Group>,
    parameters: Vec<Parameter>,
}

#[derive(Debug, Clone)]
pub struct Group {
    group_id: i8,
    group_name: String,
    description: String,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    group_id: i8,
    parameter_name: String,
    data_type: DataType,
    dimensions: Vec<u8>,
    data: ParameterData,
    description: String,
}

pub fn read_parameters_from_file(file: &str) -> Result<Parameters, C3dParseError> {
    let contents = read_c3d(file)?;

    let (parameter_start_block_index, data_start_block_index, processor_type) =
        parse_basic_info(&contents)?;

    let (_, parameter_blocks, _) = split_c3d(
        &contents,
        parameter_start_block_index,
        data_start_block_index,
    )?;

    parse_parameters(parameter_blocks, &processor_type)
}

pub fn parse_parameters(
    parameter_blocks: &[u8],
    processor_type: &ProcessorType,
) -> Result<Parameters, C3dParseError> {
    if parameter_blocks.len() < 512 {
        return Err(C3dParseError::InvalidParameterStartBlock);
    }

    let check_processor_type = match get_processor(parameter_blocks) {
        Ok(processor_type) => processor_type,
        Err(e) => return Err(e),
    };

    if check_processor_type != *processor_type {
        return Err(C3dParseError::InvalidProcessorType);
    }

    let (groups, parameters) = parse_parameter_blocks(parameter_blocks, processor_type)?;

    Ok(Parameters {
        groups,
        parameters,
    })
}

fn parse_parameter_blocks(
    parameter_blocks: &[u8],
    processor_type: &ProcessorType,
) -> Result<(Vec<Group>, Vec<Parameter>), C3dParseError> {
    if parameter_blocks.len() < 512 {
        return Err(C3dParseError::InvalidParameterStartBlock);
    }

    let mut groups = Vec::new();
    let mut parameters = Vec::new();

    let mut index = 4;

    while index != 0 {
        index = parse_next_group_or_parameter(
            &parameter_blocks,
            index,
            &mut groups,
            &mut parameters,
            processor_type,
        )?;
    }
    Ok((groups, parameters))
}

fn parse_next_group_or_parameter(
    parameter_blocks: &[u8],
    index: usize,
    groups: &mut Vec<Group>,
    parameters: &mut Vec<Parameter>,
    processor_type: &ProcessorType,
) -> Result<usize, C3dParseError> {
    let group_id = parameter_blocks[index + 1] as i8;

    if group_id == 0 {
        return Ok(0);
    } else if group_id < 0 {
        let (group, next_index) = parse_group(&parameter_blocks, index, processor_type)?;
        if next_index == 0 {
            return Ok(next_index as usize);
        }
        groups.push(group);
        Ok(next_index as usize)
    } else {
        let (parameter, next_index) = parse_parameter(&parameter_blocks, index, processor_type)?;
        if next_index == 0 {
            return Ok(next_index as usize);
        }
        parameters.push(parameter);
        Ok(next_index as usize)
    }
}

fn parse_group(
    parameter_blocks: &[u8],
    index: usize,
    processor_type: &ProcessorType,
) -> Result<(Group, u16), C3dParseError> {
    let num_chars_in_name = parameter_blocks[index] as i8;
    let group_id = parameter_blocks[index + 1] as i8;
    let group_name = parse_group_name(&parameter_blocks, index + 2, num_chars_in_name)?;
    let next_group_index_bytes = &parameter_blocks[index + 2 + num_chars_in_name.abs() as usize
        ..index + 4 + num_chars_in_name.abs() as usize];
    let next_group_index = bytes_to_i16(next_group_index_bytes, processor_type) as u16
        + 2
        + num_chars_in_name.abs() as u16
        + index as u16;
    let num_chars_in_description = parameter_blocks[index + 4 + num_chars_in_name.abs() as usize];
    let description = parse_description(
        &parameter_blocks,
        index + 5 + num_chars_in_name.abs() as usize,
        num_chars_in_description,
    )?;

    Ok((
        Group {
            group_id,
            group_name,
            description,
        },
        next_group_index as u16,
    ))
}

fn parse_group_name(
    parameter_blocks: &[u8],
    index: usize,
    num_chars_in_name: i8,
) -> Result<String, C3dParseError> {
    let mut group_name = String::new();

    for i in 0..num_chars_in_name.abs() {
        group_name.push(parameter_blocks[index + i as usize] as char);
    }

    Ok(group_name)
}

fn parse_description(
    parameter_blocks: &[u8],
    index: usize,
    num_chars_in_description: u8,
) -> Result<String, C3dParseError> {
    let mut description = String::new();

    for i in 0..num_chars_in_description {
        description.push(parameter_blocks[index + i as usize] as char);
    }

    Ok(description)
}

fn parse_parameter(
    parameter_blocks: &[u8],
    index: usize,
    processor_type: &ProcessorType,
) -> Result<(Parameter, u16), C3dParseError> {
    let num_chars_in_name = parameter_blocks[index] as i8;
    let group_id = parameter_blocks[index + 1] as i8;
    let parameter_name = parse_parameter_name(&parameter_blocks, index + 2, num_chars_in_name)?;
    let next_group_index_bytes = &parameter_blocks[index + 2 + num_chars_in_name.abs() as usize
        ..index + 4 + num_chars_in_name.abs() as usize];
    let next_group_index = bytes_to_i16(next_group_index_bytes, processor_type) as u16
        + 2
        + num_chars_in_name.abs() as u16
        + index as u16;
    let data_type =
        DataType::try_from(parameter_blocks[index + 4 + num_chars_in_name.abs() as usize] as i8)?;
    let num_dimensions = parameter_blocks[index + 5 + num_chars_in_name.abs() as usize];
    let dimensions = parse_dimensions(
        &parameter_blocks,
        index + 6 + num_chars_in_name.abs() as usize,
        num_dimensions,
    )?;
    let data = parse_data(
        &parameter_blocks,
        index + 6 + num_chars_in_name.abs() as usize + num_dimensions as usize,
        &dimensions,
        data_type,
    )?;
    let data_size = (data_type as i8).abs() as usize;
    let data_byte_size = data.len() * data_size;
    let num_chars_in_description = parameter_blocks
        [index + 6 + num_chars_in_name.abs() as usize + num_dimensions as usize + data_byte_size];
    let description = parse_description(
        &parameter_blocks,
        index + 7 + num_chars_in_name.abs() as usize + num_dimensions as usize + data_byte_size,
        num_chars_in_description,
    )?;

    Ok((
        Parameter {
            group_id,
            parameter_name,
            data_type,
            dimensions,
            data,
            description,
        },
        next_group_index,
    ))
}

fn parse_parameter_name(
    parameter_blocks: &[u8],
    index: usize,
    num_chars_in_name: i8,
) -> Result<String, C3dParseError> {
    let mut parameter_name = String::new();

    for i in 0..num_chars_in_name.abs() {
        parameter_name.push(parameter_blocks[index + i as usize] as char);
    }

    Ok(parameter_name)
}

fn parse_dimensions(
    parameter_blocks: &[u8],
    index: usize,
    num_dimensions: u8,
) -> Result<Vec<u8>, C3dParseError> {
    let mut dimensions = Vec::new();

    for i in 0..num_dimensions {
        dimensions.push(parameter_blocks[index + i as usize]);
    }

    Ok(dimensions)
}

fn parse_data(
    parameter_blocks: &[u8],
    index: usize,
    dimensions: &[u8],
    data_type: DataType,
) -> Result<Vec<u8>, C3dParseError> {
    let mut data = Vec::new();

    let mut dimensions_product = 1;
    if dimensions.len() != 0 {
        for dimension in dimensions {
            dimensions_product *= *dimension as usize;
        }
    }

    let data_size = (data_type as i8).abs() as usize;

    for i in 0..dimensions_product {
        let data_index = index + (i * data_size);
        data.push(parameter_blocks[data_index]);
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_parameters() {
        let parameters = read_parameters_from_file("res/Sample01/Eb015si.c3d");
        assert!(parameters.is_ok());
    }
    #[test]
    fn test_parse_advanced_realtime_tracking() {
        // Advanced Realtime Tracking GmbH
        assert!(read_parameters_from_file(
            "res/Sample00/Advanced Realtime Tracking GmbH/arthuman-sample.c3d"
        )
        .is_ok());
        assert!(read_parameters_from_file(
            "res/Sample00/Advanced Realtime Tracking GmbH/arthuman-sample-fingers.c3d"
        )
        .is_ok());
    }

    #[test]
    fn test_parse_codamotion() {
        // Codamotion
        assert!(read_parameters_from_file(
            "res/Sample00/Codamotion/codamotion_gaitwands_19970212.c3d"
        )
        .is_ok());
        assert!(read_parameters_from_file(
            "res/Sample00/Codamotion/codamotion_gaitwands_20150204.c3d"
        )
        .is_ok());
    }

    #[test]
    fn test_parse_cometa() {
        // Cometa
        assert!(
            read_parameters_from_file("res/Sample00/Cometa Systems/EMG Data Cometa.c3d").is_ok()
        );
    }

    #[test]
    fn test_parse_innovative_sports_training() {
        // Innovative Sports Training
        assert!(read_parameters_from_file(
            "res/Sample00/Innovative Sports Training/Gait with EMG.c3d"
        )
        .is_ok());
        assert!(read_parameters_from_file(
            "res/Sample00/Innovative Sports Training/Static Pose.c3d"
        )
        .is_ok());
    }

    #[test]
    fn test_parse_motion_analysis_corporation() {
        // Motion Analysis Corporation
        assert!(read_parameters_from_file(
            "res/Sample00/Motion Analysis Corporation/Sample_Jump2.c3d"
        )
        .is_ok());
        assert!(
            read_parameters_from_file("res/Sample00/Motion Analysis Corporation/Walk1.c3d").is_ok()
        );
    }

    #[test]
    fn test_parse_nexgen_ergonomics() {
        // NexGen Ergonomics
        assert!(read_parameters_from_file("res/Sample00/NexGen Ergonomics/test1.c3d").is_ok());
    }

    #[test]
    fn test_parse_vicon_motion_systems() {
        // Vicon Motion Systems
        assert!(
            read_parameters_from_file("res/Sample00/Vicon Motion Systems/TableTennis.c3d").is_ok()
        );
        assert!(read_parameters_from_file(
            "res/Sample00/Vicon Motion Systems/pyCGM2 lower limb CGM24 Walking01.c3d"
        )
        .is_ok());
    }
}
