use std::collections::HashMap;

use crate::c3d::C3d;
use crate::data::DataFormat;
use crate::parse::C3dParseError;
use crate::processor::{bytes_to_f32, bytes_to_i16, ProcessorType};
use ndarray::{Array, ArrayView, Axis, IxDyn, Order};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DataType {
    Char,
    Byte,
    Integer,
    Float,
}

impl From<DataType> for usize {
    fn from(data_type: DataType) -> Self {
        match data_type {
            DataType::Char => 1,
            DataType::Byte => 1,
            DataType::Integer => 2,
            DataType::Float => 4,
        }
    }
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
    Char(Array<char, IxDyn>),
    Byte(Array<u8, IxDyn>),
    Integer(Array<i16, IxDyn>),
    Float(Array<f32, IxDyn>),
}

impl From<Array<char, IxDyn>> for ParameterData {
    fn from(array: Array<char, IxDyn>) -> Self {
        ParameterData::Char(array)
    }
}

impl From<Array<u8, IxDyn>> for ParameterData {
    fn from(array: Array<u8, IxDyn>) -> Self {
        ParameterData::Byte(array)
    }
}

impl From<Array<i16, IxDyn>> for ParameterData {
    fn from(array: Array<i16, IxDyn>) -> Self {
        ParameterData::Integer(array)
    }
}

impl From<Array<f32, IxDyn>> for ParameterData {
    fn from(array: Array<f32, IxDyn>) -> Self {
        ParameterData::Float(array)
    }
}

impl ParameterData {
    pub(super) fn blank() -> Self {
        ParameterData::Char(Array::from_shape_vec(IxDyn(&[0]), vec![]).unwrap())
    }

    fn new(
        data: &[u8],
        data_type: DataType,
        dimensions: &[usize],
        processor_type: &ProcessorType,
    ) -> Result<Self, C3dParseError> {
        if data.len() % usize::from(data_type) != 0 {
            return Err(C3dParseError::InvalidParameterData);
        }
        if dimensions.iter().product::<usize>() != data.len() / usize::from(data_type) {
            return Err(C3dParseError::InvalidParameterData);
        }
        let dimensions = dimensions
            .iter()
            .map(|&x| x as usize)
            .collect::<Vec<usize>>();
        let shape = IxDyn(dimensions.as_slice());
        let array = match data_type {
            DataType::Char => {
                let data = data.iter().map(|&x| x as char).collect::<Vec<char>>();
                let array = ArrayView::<char, IxDyn>::from_shape(
                    IxDyn(vec![data.len()].as_slice()),
                    data.as_slice(),
                )
                .unwrap();
                let array = array
                    .to_shape(((shape), Order::ColumnMajor))
                    .unwrap()
                    .to_owned();
                ParameterData::Char(array)
            }
            DataType::Byte => {
                let array =
                    ArrayView::<u8, IxDyn>::from_shape(IxDyn(vec![data.len()].as_slice()), data)
                        .unwrap();
                let array = array
                    .to_shape(((shape), Order::ColumnMajor))
                    .unwrap()
                    .to_owned();
                ParameterData::from(array)
            }
            DataType::Integer => {
                let data = data
                    .chunks(2)
                    .map(|x| bytes_to_i16(x.try_into().unwrap(), processor_type))
                    .collect::<Vec<i16>>();
                let array = ArrayView::<i16, IxDyn>::from_shape(
                    IxDyn(vec![data.len()].as_slice()),
                    data.as_slice(),
                )
                .unwrap();
                let array = array
                    .to_shape(((shape), Order::ColumnMajor))
                    .unwrap()
                    .to_owned();
                ParameterData::Integer(array)
            }
            DataType::Float => {
                let data = data
                    .chunks(4)
                    .map(|x| bytes_to_f32(x.try_into().unwrap(), processor_type))
                    .collect::<Vec<f32>>();
                let array = ArrayView::<f32, IxDyn>::from_shape(
                    IxDyn(vec![data.len()].as_slice()),
                    data.as_slice(),
                )
                .unwrap();
                let array = array
                    .to_shape(((shape), Order::ColumnMajor))
                    .unwrap()
                    .to_owned();
                ParameterData::Float(array)
            }
        };

        Ok(array)
    }
}

#[derive(Debug, Clone)]
struct Parameters {
    groups: Vec<Group>,
    parameters: Vec<Parameter>,
}

impl Parameters {
    pub(super) fn blank() -> Self {
        Parameters {
            groups: vec![],
            parameters: vec![],
        }
    }

    fn get_group(&self, group_name: &str) -> Option<&Group> {
        self.groups.iter().find(|&x| x.name == group_name)
    }
    fn get_parameter(&self, group_name: &str, parameter_name: &str) -> Option<&Parameter> {
        let group = self.get_group(group_name)?;
        self.parameters
            .iter()
            .find(|&x| x.name == parameter_name && x.group_id == group.id)
    }
    pub fn get_parameter_data(
        &self,
        group_name: &str,
        parameter_name: &str,
    ) -> Option<&ParameterData> {
        let parameter = self.get_parameter(group_name, parameter_name)?;
        Some(&parameter.data)
    }

    pub fn get_data_format(&self) -> Option<DataFormat> {
        let data = self.get_parameter_data("POINT", "SCALE");
        if let Some(ParameterData::Float(array)) = data {
            let scale = array.first();
            if let Some(&scale) = scale {
                if scale < 0.0 {
                    return Some(DataFormat::Float);
                } else {
                    return Some(DataFormat::Integer);
                }
            }
        }
        None
    }

    pub fn get_num_frames(&self) -> Option<usize> {
        let data = self.get_parameter_data("POINT", "FRAMES");
        if let Some(ParameterData::Integer(array)) = data {
            let frames = array.first();
            if let Some(&frames) = frames {
                return Some(frames as usize);
            }
        }
        None
    }

    pub fn get_num_points(&self) -> Option<usize> {
        let data = self.get_parameter_data("POINT", "USED");
        if let Some(ParameterData::Integer(array)) = data {
            let points = array.first();
            if let Some(&points) = points {
                return Some(points as usize);
            }
        }
        None
    }

    pub fn get_point_labels(&self) -> Option<Vec<String>> {
        let data = self.get_parameter_data("POINT", "LABELS");
        if let Some(ParameterData::Char(array)) = data {
            let labels = array
                .axis_iter(Axis(1))
                .map(|x| x.into_iter().collect::<String>())
                .collect::<Vec<String>>();
            return Some(labels);
        }
        None
    }

    pub fn get_point_scale(&self) -> Option<f32> {
        let data = self.get_parameter_data("POINT", "SCALE");
        if let Some(ParameterData::Float(array)) = data {
            let scale = array.first();
            if let Some(&scale) = scale {
                return Some(scale);
            }
        }
        None
    }

    pub fn get_point_rate(&self) -> Option<f32> {
        let data = self.get_parameter_data("POINT", "RATE");
        if let Some(ParameterData::Float(array)) = data {
            let rate = array.first();
            if let Some(&rate) = rate {
                return Some(rate);
            }
        }
        None
    }

    pub fn get_num_analog_channels(&self) -> Option<usize> {
        let data = self.get_parameter_data("ANALOG", "USED");
        if let Some(ParameterData::Integer(array)) = data {
            let channels = array.first();
            if let Some(&channels) = channels {
                return Some(channels as usize);
            }
        }
        None
    }

    pub fn get_analog_labels(&self) -> Option<Vec<String>> {
        let data = self.get_parameter_data("ANALOG", "LABELS");
        if let Some(ParameterData::Char(array)) = data {
            let labels = array
                .axis_iter(Axis(1))
                .map(|x| x.into_iter().collect::<String>())
                .collect::<Vec<String>>();
            return Some(labels);
        }
        None
    }

    pub fn get_analog_sample_rate(&self) -> Option<f32> {
        let data = self.get_parameter_data("ANALOG", "RATE");
        if let Some(ParameterData::Float(array)) = data {
            let rate = array.first();
            if let Some(&rate) = rate {
                return Some(rate);
            }
        }
        None
    }

    pub fn get_analog_scale(&self) -> Option<f32> {
        let data = self.get_parameter_data("ANALOG", "SCALE");
        if let Some(ParameterData::Float(array)) = data {
            let scale = array.first();
            if let Some(&scale) = scale {
                return Some(scale);
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
struct Group {
    id: i8,
    name: String,
    description: String,
}

#[derive(Debug, Clone)]
struct Parameter {
    group_id: i8,
    name: String,
    pub data: ParameterData,
    description: String,
}

pub fn parse_parameter_blocks(
    parameter_blocks: &Vec<u8>,
    processor_type: &ProcessorType,
) -> Result<
    (
        HashMap<String, HashMap<String, ParameterData>>,
        HashMap<String, String>,
        HashMap<String, String>,
    ),
    C3dParseError,
> {
    if parameter_blocks.len() < 512 {
        return Err(C3dParseError::InvalidParameterStartBlock);
    }

    let mut groups: Vec<Group> = Vec::new();
    let mut parameters: Vec<Parameter> = Vec::new();

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
    let mut groups_map = HashMap::new();
    let mut group_descriptions = HashMap::new();
    let mut parameter_descriptions = HashMap::new();
    let mut temp_group_id_to_name = HashMap::new();
    for group in groups {
        temp_group_id_to_name.insert(group.id, group.name.clone());
        groups_map.insert(group.name.clone(), HashMap::new());
        group_descriptions.insert(group.name, group.description);
    }
    for parameter in parameters {
        let group_name = temp_group_id_to_name.get(&parameter.group_id);
        if let Some(group_name) = group_name {
            groups_map
                .get_mut(group_name)
                .ok_or(C3dParseError::InvalidGroupId)?
                .insert(parameter.name.clone(), parameter.data);
            parameter_descriptions.insert(parameter.name, parameter.description);

        }
        else {
            return Err(C3dParseError::InvalidGroupId);
        }
    }
    Ok((groups_map, group_descriptions, parameter_descriptions))
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
        groups.push(group);
        Ok(next_index as usize)
    } else {
        let (parameter, next_index) = parse_parameter(&parameter_blocks, index, processor_type)?;
        parameters.push(parameter);
        Ok(next_index as usize)
    }
}

fn parse_group(
    parameter_blocks: &[u8],
    index: usize,
    processor_type: &ProcessorType,
) -> Result<(Group, u16), C3dParseError> {
    let mut i = index;
    let num_chars_in_name = parameter_blocks[i] as i8;
    i += 1;
    let id = (parameter_blocks[i] as i8).abs();
    i += 1;
    let name = parse_group_name(&parameter_blocks, i, num_chars_in_name)?;
    i += num_chars_in_name.abs() as usize;
    let next_group_index_bytes = &parameter_blocks[i..i + 2];
    let next_group_index =
        bytes_to_i16(next_group_index_bytes.try_into().unwrap(), processor_type) as u16 + i as u16;
    i += 2;
    let num_chars_in_description = parameter_blocks[i];
    i += 1;
    let description = parse_description(&parameter_blocks, i, num_chars_in_description)?;

    Ok((
        Group {
            id,
            name,
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
    let mut i = index;
    let num_chars_in_name = parameter_blocks[i] as i8;
    i += 1;
    let group_id = parameter_blocks[i] as i8;
    i += 1;
    let name = parse_parameter_name(&parameter_blocks, i, num_chars_in_name)?;
    i += num_chars_in_name.abs() as usize;
    let next_group_index_bytes = &parameter_blocks[i..i + 2];
    let next_group_index =
        bytes_to_i16(next_group_index_bytes.try_into().unwrap(), processor_type) as u16 + i as u16;
    i += 2;
    let data_type = DataType::try_from(parameter_blocks[i] as i8)?;
    i += 1;
    let num_dimensions = parameter_blocks[i];
    i += 1;
    let dimensions = parse_dimensions(&parameter_blocks, i, num_dimensions)?;
    i += num_dimensions as usize;
    let (data, data_byte_size) =
        parse_data(&parameter_blocks, i, &dimensions, data_type, processor_type)?;
    i += data_byte_size;
    let num_chars_in_description = parameter_blocks[i];
    i += 1;
    let description = parse_description(&parameter_blocks, i, num_chars_in_description)?;

    Ok((
        Parameter {
            group_id,
            name,
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
    dimensions: &Vec<u8>,
    data_type: DataType,
    processor_type: &ProcessorType,
) -> Result<(ParameterData, usize), C3dParseError> {
    let dimensions_product = &dimensions
        .clone()
        .iter()
        .map(|x| *x as usize)
        .product::<usize>();

    let data_byte_size = dimensions_product * usize::from(data_type);

    let bytes: &[u8] = &parameter_blocks[index..index + data_byte_size];
    let dimensions: &[usize] = &dimensions
        .iter()
        .map(|x| *x as usize)
        .collect::<Vec<usize>>();

    Ok((
        ParameterData::new(bytes, data_type, dimensions, processor_type)?,
        data_byte_size,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_parameters() {
        let parameters = C3d::parameters("res/Sample01/Eb015si.c3d");
        assert!(parameters.is_ok());
    }
    #[test]
    fn test_parse_advanced_realtime_tracking() {
        // Advanced Realtime Tracking GmbH
        assert!(C3d::parameters(
            "res/Sample00/Advanced Realtime Tracking GmbH/arthuman-sample.c3d"
        )
        .is_ok());
        assert!(C3d::parameters(
            "res/Sample00/Advanced Realtime Tracking GmbH/arthuman-sample-fingers.c3d"
        )
        .is_ok());
    }

    #[test]
    fn test_parse_codamotion() {
        // Codamotion
        assert!(
            C3d::parameters("res/Sample00/Codamotion/codamotion_gaitwands_19970212.c3d").is_ok()
        );
        assert!(
            C3d::parameters("res/Sample00/Codamotion/codamotion_gaitwands_20150204.c3d").is_ok()
        );
    }

    #[test]
    fn test_parse_cometa() {
        // Cometa
        assert!(C3d::parameters("res/Sample00/Cometa Systems/EMG Data Cometa.c3d").is_ok());
    }

    #[test]
    fn test_parse_innovative_sports_training() {
        // Innovative Sports Training
        assert!(
            C3d::parameters("res/Sample00/Innovative Sports Training/Gait with EMG.c3d").is_ok()
        );
        assert!(C3d::parameters("res/Sample00/Innovative Sports Training/Static Pose.c3d").is_ok());
    }

    #[test]
    fn test_parse_motion_analysis_corporation() {
        // Motion Analysis Corporation
        assert!(
            C3d::parameters("res/Sample00/Motion Analysis Corporation/Sample_Jump2.c3d").is_ok()
        );
        assert!(C3d::parameters("res/Sample00/Motion Analysis Corporation/Walk1.c3d").is_ok());
    }

    #[test]
    fn test_parse_nexgen_ergonomics() {
        // NexGen Ergonomics
        assert!(C3d::parameters("res/Sample00/NexGen Ergonomics/test1.c3d").is_ok());
    }

    #[test]
    fn test_parse_vicon_motion_systems() {
        // Vicon Motion Systems
        assert!(C3d::parameters("res/Sample00/Vicon Motion Systems/TableTennis.c3d").is_ok());
        assert!(C3d::parameters(
            "res/Sample00/Vicon Motion Systems/pyCGM2 lower limb CGM24 Walking01.c3d"
        )
        .is_ok());
    }
}
