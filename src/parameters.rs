use crate::processor::Processor;
use crate::{C3dIoError, C3dParseError};
use grid::Grid;
use std::collections::HashMap;

/// The parameters of a C3D file are stored in a `Parameters` struct.
/// Each group of parameters is stored in a separate struct.
/// The `raw_parameters` field is a `HashMap` of `HashMap`s.
/// The first key is the group name, and the second key is the parameter name.
/// The value is a tuple of the parameter data and the description.
#[derive(Debug, Clone, Default)]
pub struct Parameters {
    parameters: HashMap<String, (String, HashMap<String, Parameter>)>,
}

impl ToString for Parameters {
    fn to_string(&self) -> String {
        let mut string = String::new();
        for (group, (group_description, parameters)) in self.parameters.iter() {
            string.push_str(&format!(
                "Group: {}\nDescription: {}\n",
                group, group_description
            ));
            for (parameter_name, parameter) in parameters.iter() {
                string.push_str(&format!(
                    "Parameter: {}\nDescription: {}\n",
                    parameter_name, parameter.description
                ));
                string.push_str(&format!("Dimensions: {:?}\n", parameter.dimensions));
                string.push_str(&format!("Data: {:?}\n", parameter.data));
            }
        }
        string
    }
}

impl Parameters {
    pub fn new() -> Parameters {
        Parameters::default()
    }

    pub(crate) fn parse_parameter_blocks(
        parameter_blocks: &Vec<u8>,
        processor: &Processor,
    ) -> Result<Self, C3dParseError> {
        Parameters::from_bytes(parameter_blocks, processor)
    }

    pub(crate) fn from_bytes(
        parameter_blocks: &Vec<u8>,
        processor: &Processor,
    ) -> Result<Self, C3dParseError> {
        Ok(Parameters {
            parameters: parse_parameters(parameter_blocks, processor)?,
        })
    }

    pub(crate) fn write_groups(
        &self,
        processor: &Processor,
    ) -> Result<(Vec<u8>, HashMap<String, usize>), C3dIoError> {
        let mut bytes = Vec::new();
        let mut group_names_to_ids = HashMap::new();
        let mut group_id = 1;
        for (i, (group, (group_description, parameters))) in self.parameters.iter().enumerate() {
            if group.len() > 127 {
                return Err(C3dIoError::GroupNameTooLong(group.clone()));
            }
            bytes.push(group.len() as u8);
            bytes.push(-(group_id as i8) as u8);
            if !group.is_ascii() {
                return Err(C3dIoError::GroupNameNotAscii(group.clone()));
            }
            bytes.extend(group.to_ascii_uppercase().as_bytes());
            if group_description.as_bytes().len() > 255 {
                return Err(C3dIoError::GroupDescriptionTooLong(
                    group_description.clone(),
                ));
            }
            let bytes_to_end = processor.u16_to_bytes(group_description.len() as u16 + 2);
            bytes.extend(bytes_to_end);
            bytes.extend(group_description.as_bytes());

            group_names_to_ids.insert(group.clone(), group_id);
            group_id += 1;
        }
        Ok((bytes, group_names_to_ids))
    }

    pub(crate) fn write_parameters(
        &self,
        processor: &Processor,
        group_names_to_ids: &HashMap<String, usize>,
    ) -> Result<Vec<u8>, C3dIoError> {
        let mut bytes = Vec::new();
        for (i, (group, (_, parameters))) in self.parameters.iter().enumerate() {
            let group_id = group_names_to_ids.get(group).unwrap();
            for (j, (parameter_name, parameter)) in parameters.iter().enumerate() {
                bytes.extend(parameter.write(
                    processor,
                    parameter_name.clone(),
                    *group_id,
                    j == parameters.len() - 1 && i == self.parameters.len() - 1,
                )?);
            }
        }
        Ok(bytes)
    }

    pub fn get(&self, group: &str, parameter: &str) -> Option<&Parameter> {
        self.parameters
            .get(group)
            .and_then(|(_, group)| group.get(parameter))
    }

    pub fn get_mut(&mut self, group: &str, parameter: &str) -> Option<&mut Parameter> {
        self.parameters
            .get_mut(group)
            .and_then(|(_, group)| group.get_mut(parameter))
    }

    pub fn get_or_err(&self, group: &str, parameter: &str) -> Result<&Parameter, C3dParseError> {
        self.get(group, parameter)
            .ok_or(C3dParseError::ParameterNotFound(
                group.to_string(),
                parameter.to_string(),
            ))
    }

    pub fn get_mut_or_err(
        &mut self,
        group: &str,
        parameter: &str,
    ) -> Result<&mut Parameter, C3dParseError> {
        self.get_mut(group, parameter)
            .ok_or(C3dParseError::ParameterNotFound(
                group.to_string(),
                parameter.to_string(),
            ))
    }

    pub fn insert(&mut self, group: &str, parameter: &str, mut value: Parameter) {
        value.name = parameter.to_string();
        self.parameters
            .entry(group.to_string())
            .or_insert((String::new(), HashMap::new()))
            .1
            .insert(parameter.to_string(), value);
    }

    pub fn remove(&mut self, group: &str, parameter: &str) -> Option<Parameter> {
        self.parameters
            .get_mut(group)
            .and_then(|(_, group)| group.remove(parameter))
    }

    pub fn remove_or_err(
        &mut self,
        group: &str,
        parameter: &str,
    ) -> Result<Parameter, C3dParseError> {
        self.remove(group, parameter)
            .ok_or(C3dParseError::ParameterNotFound(
                group.to_string(),
                parameter.to_string(),
            ))
    }

    pub fn contains(&self, group: &str, parameter: &str) -> bool {
        self.parameters
            .get(group)
            .and_then(|(_, group)| group.get(parameter))
            .is_some()
    }

    pub fn get_group(&self, group: &str) -> Option<&HashMap<String, Parameter>> {
        self.parameters.get(group).map(|(_, group)| group)
    }

    pub fn get_group_description(&self, group: &str) -> Option<&String> {
        self.parameters
            .get(group)
            .map(|(group_description, _)| group_description)
    }

    pub fn get_group_description_mut(&mut self, group: &str) -> Option<&mut String> {
        self.parameters
            .get_mut(group)
            .map(|(group_description, _)| group_description)
    }

    pub fn insert_group(&mut self, group_name: &str, description: String) {
        self.parameters
            .insert(group_name.to_string(), (description, HashMap::new()));
    }

    pub fn num_groups(&self) -> usize {
        self.parameters.len()
    }

    pub fn num_parameters(&self, group: &str) -> Option<usize> {
        self.parameters.get(group).map(|(_, group)| group.len())
    }

    pub fn groups(&self) -> Vec<&String> {
        self.parameters.keys().collect()
    }

    pub fn parameters(&self, group: &str) -> Option<Vec<&Parameter>> {
        self.parameters
            .get(group)
            .map(|(_, group)| group.values().collect())
    }
}

impl PartialEq for Parameters {
    fn eq(&self, other: &Self) -> bool {
        //if self.parameters.len() != other.parameters.len() {
        //    return false;
        //}
        //for (group, (group_description, parameters)) in self.parameters.iter() {
        //    if !other.parameters.contains_key(group) {
        //        return false;
        //    }
        //    let other_group = other.parameters.get(group).unwrap();
        //    if group_description != &other_group.0 {
        //        return false;
        //    }
        //    if parameters.len() != other_group.1.len() {
        //        return false;
        //    }
        //    for (parameter_name, parameter) in parameters.iter() {
        //        if !other_group.1.contains_key(parameter_name) {
        //            return false;
        //        }
        //        let other_parameter = other_group.1.get(parameter_name).unwrap();
        //        if parameter != other_parameter {
        //            return false;
        //        }
        //    }
        //}
        true
    }
}

impl AsRef<Parameter> for Parameter {
    fn as_ref(&self) -> &Parameter {
        self
    }
}

impl TryFrom<&Parameter> for u16 {
    type Error = C3dParseError;
    fn try_from(parameter: &Parameter) -> Result<Self, Self::Error> {
        match &parameter.data {
            ParameterData::Integer(data) => {
                if data.len() == 1 {
                    Ok(data[0] as u16)
                } else {
                    Err(C3dParseError::InvalidData(
                        parameter.clone(),
                        "u16".to_string(),
                    ))
                }
            }
            _ => Err(C3dParseError::InvalidData(
                parameter.clone(),
                "u16".to_string(),
            )),
        }
    }
}

impl TryFrom<&Parameter> for [u16; 2] {
    type Error = C3dParseError;
    fn try_from(parameter: &Parameter) -> Result<Self, Self::Error> {
        match &parameter.data {
            ParameterData::Integer(data) => {
                if data.len() == 2 {
                    Ok([data[0] as u16, data[1] as u16])
                } else {
                    Err(C3dParseError::InvalidData(
                        parameter.clone(),
                        "u162".to_string(),
                    ))
                }
            }
            _ => Err(C3dParseError::InvalidData(
                parameter.clone(),
                "u162".to_string(),
            )),
        }
    }
}

impl TryFrom<&Parameter> for Vec<u16> {
    type Error = C3dParseError;
    fn try_from(parameter: &Parameter) -> Result<Self, Self::Error> {
        match &parameter.data {
            ParameterData::Integer(data) => {
                if parameter.dimensions.len() == 1 {
                    Ok(data.iter().map(|x| *x as u16).collect())
                } else {
                    Err(C3dParseError::InvalidData(
                        parameter.clone(),
                        "Vec<u16>".to_string(),
                    ))
                }
            }
            _ => Err(C3dParseError::InvalidData(
                parameter.clone(),
                "Vec<u16>".to_string(),
            )),
        }
    }
}

impl TryFrom<&Parameter> for i16 {
    type Error = C3dParseError;
    fn try_from(parameter: &Parameter) -> Result<Self, Self::Error> {
        match &parameter.data {
            ParameterData::Integer(data) => {
                if data.len() == 1 {
                    Ok(data[0])
                } else {
                    Err(C3dParseError::InvalidData(
                        parameter.clone(),
                        "i16".to_string(),
                    ))
                }
            }
            _ => Err(C3dParseError::InvalidData(
                parameter.clone(),
                "i16".to_string(),
            )),
        }
    }
}

impl TryFrom<&Parameter> for Vec<i16> {
    type Error = C3dParseError;
    fn try_from(parameter: &Parameter) -> Result<Self, Self::Error> {
        match &parameter.data {
            ParameterData::Integer(data) => {
                if parameter.dimensions.len() == 1 {
                    Ok(data.iter().map(|&x| x as i16).collect())
                } else {
                    Err(C3dParseError::InvalidData(
                        parameter.clone(),
                        "Vec<i16>".to_string(),
                    ))
                }
            }
            _ => Err(C3dParseError::InvalidData(
                parameter.clone(),
                "Vec<i16>".to_string(),
            )),
        }
    }
}

impl TryFrom<&Parameter> for Grid<i16> {
    type Error = C3dParseError;
    fn try_from(parameter: &Parameter) -> Result<Self, Self::Error> {
        let dimensions = parameter.dimensions.clone();
        match &parameter.data {
            ParameterData::Integer(data) => {
                if dimensions.len() == 2 {
                    Ok(Grid::from_vec(data.clone(), dimensions[0] as usize))
                } else if dimensions.len() == 1 {
                    Ok(Grid::from_vec(data.clone(), 1))
                } else {
                    Err(C3dParseError::InvalidData(
                        parameter.clone(),
                        "Grid<i16>".to_string(),
                    ))
                }
            }
            _ => Err(C3dParseError::InvalidData(
                parameter.clone(),
                "Grid<i16>".to_string(),
            )),
        }
    }
}

impl TryFrom<&Parameter> for f32 {
    type Error = C3dParseError;
    fn try_from(parameter: &Parameter) -> Result<Self, Self::Error> {
        match &parameter.data {
            ParameterData::Float(data) => {
                if data.len() == 1 {
                    Ok(data[0])
                } else {
                    Err(C3dParseError::InvalidData(
                        parameter.clone(),
                        "f32".to_string(),
                    ))
                }
            }
            _ => Err(C3dParseError::InvalidData(
                parameter.clone(),
                "f32".to_string(),
            )),
        }
    }
}

impl TryFrom<&Parameter> for Vec<f32> {
    type Error = C3dParseError;
    fn try_from(parameter: &Parameter) -> Result<Self, Self::Error> {
        match &parameter.data {
            ParameterData::Float(data) => {
                if parameter.dimensions.len() == 1 {
                    Ok(data.iter().map(|&x| x as f32).collect())
                } else {
                    Err(C3dParseError::InvalidData(
                        parameter.clone(),
                        "Vec<f32>".to_string(),
                    ))
                }
            }
            _ => Err(C3dParseError::InvalidData(
                parameter.clone(),
                "Vec<f32>".to_string(),
            )),
        }
    }
}

impl TryFrom<&Parameter> for Grid<f32> {
    type Error = C3dParseError;
    fn try_from(parameter: &Parameter) -> Result<Self, Self::Error> {
        let dimensions = parameter.dimensions.clone();
        match &parameter.data {
            ParameterData::Float(data) => {
                if dimensions.len() == 2 {
                    Ok(Grid::from_vec(data.clone(), dimensions[1] as usize))
                } else if dimensions.len() == 1 {
                    Ok(Grid::from_vec(data.clone(), 1))
                } else {
                    Err(C3dParseError::InvalidData(
                        parameter.clone(),
                        "Grid<f32>".to_string(),
                    ))
                }
            }
            _ => Err(C3dParseError::InvalidData(
                parameter.clone(),
                "Grid<f32>".to_string(),
            )),
        }
    }
}

impl TryFrom<&Parameter> for String {
    type Error = C3dParseError;
    fn try_from(parameter: &Parameter) -> Result<Self, Self::Error> {
        match &parameter.data {
            ParameterData::Char(data) => {
                let mut string = String::new();
                for c in 0..data.len() {
                    string.push(data[c]);
                }
                Ok(string.trim().to_string())
            }
            _ => Err(C3dParseError::InvalidData(
                parameter.clone(),
                "String".to_string(),
            )),
        }
    }
}

impl TryFrom<&Parameter> for Vec<String> {
    type Error = C3dParseError;
    fn try_from(parameter: &Parameter) -> Result<Self, Self::Error> {
        match &parameter.data {
            ParameterData::Char(data) => {
                if parameter.dimensions.len() == 2 || parameter.dimensions.len() == 1 {
                    let mut strings = Vec::new();
                    let num_strings = match parameter.dimensions.len() == 1 {
                        true => 1,
                        false => parameter.dimensions[1] as usize,
                    };
                    for row in 0..num_strings {
                        let mut string = String::new();
                        for c in 0..parameter.dimensions[0] as usize {
                            string.push(data[row * parameter.dimensions[0] as usize + c]);
                        }
                        strings.push(string.trim().to_string());
                    }
                    Ok(strings)
                } else if parameter.dimensions.len() == 0 {
                    Ok(Vec::new())
                } else {
                    Err(C3dParseError::InvalidData(
                        parameter.clone(),
                        "Vec<String>".to_string(),
                    ))
                }
            }
            _ => Err(C3dParseError::InvalidData(
                parameter.clone(),
                "Vec<String>".to_string(),
            )),
        }
    }
}

impl TryFrom<&Parameter> for [char; 4] {
    type Error = C3dParseError;
    fn try_from(parameter: &Parameter) -> Result<Self, Self::Error> {
        match &parameter.data {
            ParameterData::Char(data) => {
                let mut chars = [' '; 4];
                for (i, c) in data.iter().enumerate() {
                    if i >= 4 {
                        break;
                    }
                    chars[i] = *c;
                }
                Ok(chars)
            }
            _ => Err(C3dParseError::InvalidData(
                parameter.clone(),
                "char4".to_string(),
            )),
        }
    }
}

impl TryFrom<&Parameter> for Vec<[char; 4]> {
    type Error = C3dParseError;
    fn try_from(parameter: &Parameter) -> Result<Self, Self::Error> {
        match &parameter.data {
            ParameterData::Char(data) => {
                if parameter.dimensions.len() == 2 && parameter.dimensions[1] == 4 {
                    let mut chars = Vec::new();
                    for row in 0..data.len() % 4 {
                        let mut chars_row = [' '; 4];
                        for i in 0..4 {
                            chars_row[i] = data[row * 4 + i];
                        }
                        chars.push(chars_row);
                    }
                    Ok(chars)
                } else {
                    Err(C3dParseError::InvalidData(
                        parameter.clone(),
                        "Vec<[char; 4]>".to_string(),
                    ))
                }
            }
            _ => Err(C3dParseError::InvalidData(
                parameter.clone(),
                "Vec<[char; 4]>".to_string(),
            )),
        }
    }
}

impl TryFrom<&Parameter> for [char; 2] {
    type Error = C3dParseError;
    fn try_from(parameter: &Parameter) -> Result<Self, Self::Error> {
        match &parameter.data {
            ParameterData::Char(data) => {
                let mut chars = [' '; 2];
                for (i, c) in data.iter().enumerate() {
                    if i >= 2 {
                        break;
                    }
                    chars[i] = *c;
                }
                Ok(chars)
            }
            _ => Err(C3dParseError::InvalidData(
                parameter.clone(),
                "char2".to_string(),
            )),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DataType {
    Char = -1,
    Byte = 1,
    Integer = 2,
    Float = 4,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub(crate) name: String,
    pub description: String,
    pub dimensions: Vec<u8>,
    pub data: ParameterData,
}

impl Parameter {
    pub fn new(
        description: String,
        dimensions: Vec<u8>,
        data: ParameterData,
    ) -> Result<Self, C3dParseError> {
        let data_length = match &data {
            ParameterData::Char(data) => data.len(),
            ParameterData::Byte(data) => data.len(),
            ParameterData::Integer(data) => data.len() * 2,
            ParameterData::Float(data) => data.len() * 4,
        };
        let mut dimensions = dimensions;
        dimensions.retain(|&x| x != 0);
        dimensions.retain(|&x| x != 1);
        if dimensions.len() == 0 {
            dimensions.push(1);
        }
        if dimensions.iter().map(|&x| x as usize).product::<usize>() != data_length {
            return Err(C3dParseError::InvalidParameterData);
        }
        Ok(Parameter {
            name: String::new(),
            description,
            dimensions,
            data,
        })
    }

    pub fn empty_bytes() -> Self {
        Parameter {
            name: String::new(),
            description: String::new(),
            dimensions: Vec::new(),
            data: ParameterData::Byte(Vec::new()),
        }
    }

    pub fn empty_chars() -> Self {
        Parameter {
            name: String::new(),
            description: String::new(),
            dimensions: Vec::new(),
            data: ParameterData::Char(Vec::new()),
        }
    }

    pub fn empty_integers() -> Self {
        Parameter {
            name: String::new(),
            description: String::new(),
            dimensions: Vec::new(),
            data: ParameterData::Integer(Vec::new()),
        }
    }

    pub fn empty_floats() -> Self {
        Parameter {
            name: String::new(),
            description: String::new(),
            dimensions: Vec::new(),
            data: ParameterData::Float(Vec::new()),
        }
    }

    pub fn byte(data: u8) -> Self {
        Parameter {
            name: String::new(),
            description: String::new(),
            dimensions: vec![1],
            data: ParameterData::Byte(vec![data]),
        }
    }

    pub fn char(data: char) -> Self {
        Parameter {
            name: String::new(),
            description: String::new(),
            dimensions: vec![1],
            data: ParameterData::Char(vec![data]),
        }
    }

    pub fn integer(data: i16) -> Self {
        Parameter {
            name: String::new(),
            description: String::new(),
            dimensions: vec![1],
            data: ParameterData::Integer(vec![data]),
        }
    }

    pub fn float(data: f32) -> Self {
        Parameter {
            name: String::new(),
            description: String::new(),
            dimensions: vec![1],
            data: ParameterData::Float(vec![data]),
        }
    }

    pub fn bytes(data: Vec<u8>) -> Self {
        Parameter {
            name: String::new(),
            description: String::new(),
            dimensions: vec![data.len() as u8],
            data: ParameterData::Byte(data),
        }
    }

    pub fn chars(data: Vec<char>) -> Result<Self, C3dIoError> {
        if data.len() == 0 {
            return Err(C3dIoError::InvalidParameterDimensions(
                "chars".to_string(),
            ));
        }
        Ok(Parameter {
            name: String::new(),
            description: String::new(),
            dimensions: vec![data.len() as u8],
            data: ParameterData::Char(data),
        })
    }

    pub fn string(data: String) -> Result<Parameter, C3dIoError> {
        if data.len() == 0 {
            return Err(C3dIoError::InvalidParameterDimensions(
                "string".to_string(),
            ));
        }
        let chars = data.chars().collect::<Vec<char>>();
        Parameter::chars(chars)
    }

    pub fn integers(data: Vec<i16>) -> Result<Self, C3dIoError> {
        if data.len() == 0 {
            return Err(C3dIoError::InvalidParameterDimensions(
                "integers".to_string(),
            ));
        }
        Ok(Parameter {
            name: String::new(),
            description: String::new(),
            dimensions: vec![data.len() as u8],
            data: ParameterData::Integer(data),
        })
    }

    pub fn floats(data: Vec<f32>) -> Result<Parameter, C3dIoError> {
        if data.len() == 0 {
            return Err(C3dIoError::InvalidParameterDimensions(
                "floats".to_string(),
            ));
        }
        Ok(Parameter {
            name: String::new(),
            description: String::new(),
            dimensions: vec![data.len() as u8],
            data: ParameterData::Float(data),
        })
    }

    pub fn byte_grid(data: Grid<u8>) -> Self {
        Parameter {
            name: String::new(),
            description: String::new(),
            dimensions: vec![data.rows() as u8, data.cols() as u8],
            data: ParameterData::Byte(data.into_vec()),
        }
    }

    pub fn strings(mut data: Vec<String>) -> Self {
        if data.len() == 0 {
            data.push(String::new());
        }
        let mut max_length = 1; //no zero length strings allowed
        for string in &data {
            if string.len() > max_length {
                max_length = string.len();
            }
        }
        let mut char_grid = Grid::new(0, max_length);
        for string in data {
            let mut chars = string.chars().collect::<Vec<char>>();
            chars.resize(max_length, ' ');
            char_grid.push_row(chars);
        }
        Parameter::char_grid(char_grid)
    }

    pub fn char_grid(data: Grid<char>) -> Self {
        Parameter {
            name: String::new(),
            description: String::new(),
            dimensions: vec![data.cols() as u8, data.rows() as u8],
            data: ParameterData::Char(data.into_vec()),
        }
    }

    pub fn integer_grid(data: Grid<i16>) -> Self {
        Parameter {
            name: String::new(),
            description: String::new(),
            dimensions: vec![data.cols() as u8, data.rows() as u8],
            data: ParameterData::Integer(data.into_vec()),
        }
    }

    pub fn float_grid(data: Grid<f32>) -> Self {
        Parameter {
            name: String::new(),
            description: String::new(),
            dimensions: vec![data.cols() as u8, data.rows() as u8],
            data: ParameterData::Float(data.into_vec()),
        }
    }

    pub(crate) fn write(
        &self,
        processor: &Processor,
        parameter_name: String,
        group_id: usize,
        last_parameter: bool,
    ) -> Result<Vec<u8>, C3dIoError> {
        let mut bytes = Vec::new();
        if parameter_name.len() > 127 {
            return Err(C3dIoError::ParameterNameTooLong(parameter_name.clone()));
        }
        bytes.push(parameter_name.len() as u8);
        bytes.push(group_id.clone() as u8);
        if !parameter_name.is_ascii() {
            return Err(C3dIoError::ParameterNameNotAscii(parameter_name.clone()));
        }
        bytes.extend(parameter_name.to_ascii_uppercase().as_bytes());
        if last_parameter {
            bytes.extend(processor.u16_to_bytes(0));
        } else {
            let mut bytes_to_end = 4;
            bytes_to_end += self.dimensions.len();
            match &self.data {
                ParameterData::Char(data) => {
                    bytes_to_end += data.len();
                }
                ParameterData::Byte(data) => {
                    bytes_to_end += data.len();
                }
                ParameterData::Integer(data) => {
                    bytes_to_end += data.len() * 2;
                }
                ParameterData::Float(data) => {
                    bytes_to_end += data.len() * 4;
                }
            }
            bytes_to_end += 1;
            if self.description.as_bytes().len() > 255 {
                return Err(C3dIoError::ParameterDescriptionTooLong(
                    parameter_name.clone(),
                ));
            }
            bytes_to_end += self.description.len();
            bytes.extend(processor.u16_to_bytes(bytes_to_end as u16));
        }
        match &self.data {
            ParameterData::Char(_) => {
                bytes.push(DataType::Char as u8);
            }
            ParameterData::Byte(_) => {
                bytes.push(DataType::Byte as u8);
            }
            ParameterData::Integer(_) => {
                bytes.push(DataType::Integer as u8);
            }
            ParameterData::Float(_) => {
                bytes.push(DataType::Float as u8);
            }
        }
        bytes.push(self.dimensions.len() as u8);
        for dimension in &self.dimensions {
            if dimension > &255 {
                return Err(C3dIoError::InvalidParameterDimensions(
                    parameter_name.clone(),
                ));
            }
        }
        bytes.extend(&self.dimensions);
        match &self.data {
            ParameterData::Char(data) => {
                bytes.extend(data.iter().map(|&x| x as u8));
            }
            ParameterData::Byte(data) => {
                bytes.extend(data);
            }
            ParameterData::Integer(data) => {
                bytes.extend(data.iter().flat_map(|&x| processor.i16_to_bytes(x)));
            }
            ParameterData::Float(data) => {
                bytes.extend(data.iter().flat_map(|&x| processor.f32_to_bytes(x)));
            }
        }
        bytes.push(self.description.len() as u8);
        bytes.extend(self.description.as_bytes());
        Ok(bytes)
    }
}

/// All parameter data is stored as a vector of bytes, but the data type and dimensions of the data
/// are also stored in the file. This struct stores the data type and dimensions, and provides
/// methods to convert the data to a more useful format.
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterData {
    Char(Vec<char>),
    Byte(Vec<u8>),
    Integer(Vec<i16>),
    Float(Vec<f32>),
}

impl ParameterData {
    pub(crate) fn new(
        data: &[u8],
        dimensions: &Vec<u8>,
        data_type: DataType,
        processor: &Processor,
    ) -> Result<Self, C3dParseError> {
        if data.len() % usize::from(data_type) != 0 {
            return Err(C3dParseError::InvalidParameterData);
        }
        let dimensions = dimensions
            .iter()
            .map(|&x| x as usize)
            .collect::<Vec<usize>>();
        if dimensions.iter().product::<usize>() != data.len() / usize::from(data_type) {
            return Err(C3dParseError::InvalidParameterData);
        }
        let array = match data_type {
            DataType::Char => {
                let array = Vec::from(data.iter().map(|&x| x as char).collect::<Vec<char>>());
                ParameterData::Char(array)
            }
            DataType::Byte => {
                let array = Vec::from(data.iter().map(|&x| x).collect::<Vec<u8>>());
                ParameterData::Byte(array)
            }
            DataType::Integer => {
                let array = Vec::from(
                    data.chunks(2)
                        .map(|x| processor.i16(x.try_into().unwrap()))
                        .collect::<Vec<i16>>(),
                );
                ParameterData::Integer(array)
            }
            DataType::Float => {
                let array = Vec::from(
                    data.chunks(4)
                        .map(|x| processor.f32(x.try_into().unwrap()))
                        .collect::<Vec<f32>>(),
                );
                ParameterData::Float(array)
            }
        };

        Ok(array)
    }
}

fn parse_parameters(
    parameter_blocks: &Vec<u8>,
    processor: &Processor,
) -> Result<HashMap<String, (String, HashMap<String, Parameter>)>, C3dParseError> {
    if parameter_blocks.len() < 512 {
        return Err(C3dParseError::InvalidParameterStartBlock);
    }

    let mut groups: Vec<ParsedGroup> = Vec::new();
    let mut parameters: Vec<ParsedParameter> = Vec::new();

    let mut index = 4; // start after parameter header

    while index != 0 {
        index = parse_next_group_or_parameter(
            &parameter_blocks,
            index,
            &mut groups,
            &mut parameters,
            processor,
        )?;
    }
    let mut groups_map = HashMap::new();
    let mut temp_group_id_to_name = HashMap::new();
    for group in groups {
        temp_group_id_to_name.insert(group.id, group.name.clone());
        groups_map.insert(group.name.clone(), (group.description, HashMap::new()));
    }
    insert_missing_required_groups(&mut groups_map, &mut temp_group_id_to_name)?;
    for parameter in parameters {
        let group_name = match temp_group_id_to_name.contains_key(&parameter.group_id) {
            true => temp_group_id_to_name
                .get(&parameter.group_id)
                .unwrap()
                .clone(),
            false => {
                temp_group_id_to_name.insert(parameter.group_id, parameter.group_id.to_string());
                groups_map.insert(
                    parameter.group_id.to_string(),
                    ("".to_string(), HashMap::new()),
                );
                parameter.group_id.to_string()
            }
        };
        let name = parameter.name.clone();
        let parameter = Parameter {
            name: parameter.name,
            description: parameter.description,
            dimensions: parameter.dimensions,
            data: parameter.data,
        };
        groups_map
            .get_mut(&group_name)
            .ok_or(C3dParseError::InvalidGroupId)?
            .1
            .insert(name, parameter);
    }
    Ok(groups_map)
}

fn parse_next_group_or_parameter(
    parameter_blocks: &Vec<u8>,
    index: usize,
    groups: &mut Vec<ParsedGroup>,
    parameters: &mut Vec<ParsedParameter>,
    processor: &Processor,
) -> Result<usize, C3dParseError> {
    if index + 1 >= parameter_blocks.len() {
        return Ok(0);
        //return Err(C3dParseError::InvalidNextParameter);
    }
    let group_id = parameter_blocks[index + 1] as i8;

    if group_id == 0 {
        return Ok(0);
    } else if group_id < 0 {
        let (group, next_index) = parse_group(&parameter_blocks, index, processor)?;
        groups.push(group);
        Ok(next_index as usize)
    } else {
        let (parameter, next_index) = parse_parameter(&parameter_blocks, index, processor)?;
        parameters.push(parameter);
        Ok(next_index as usize)
    }
}

fn parse_group(
    parameter_blocks: &Vec<u8>,
    index: usize,
    processor: &Processor,
) -> Result<(ParsedGroup, usize), C3dParseError> {
    let mut i = index;
    let num_chars_in_name = parameter_blocks[i] as i8;
    i += 1;
    let id = (parameter_blocks[i] as i8).abs();
    i += 1;
    let name = parse_group_name(&parameter_blocks, i, num_chars_in_name)?;
    i += num_chars_in_name.abs() as usize;
    let next_group_index_bytes = &parameter_blocks[i..i + 2];
    let next_group_index =
        processor.u16(next_group_index_bytes.try_into().unwrap()) as usize + i as usize;
    i += 2;
    let num_chars_in_description = parameter_blocks[i];
    i += 1;
    let description = parse_description(&parameter_blocks, i, num_chars_in_description)?;

    Ok((
        ParsedGroup {
            id,
            name,
            description,
        },
        next_group_index,
    ))
}

fn parse_group_name(
    parameter_blocks: &Vec<u8>,
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
    parameter_blocks: &Vec<u8>,
    index: usize,
    num_chars_in_description: u8,
) -> Result<String, C3dParseError> {
    let mut description = Vec::new();
    for i in 0..num_chars_in_description {
        description.push(parameter_blocks[index + i as usize]);
    }
    let utf = String::from_utf8(description.clone());
    match utf {
        Ok(utf) => Ok(utf),
        //Err(_) => Err(C3dParseError::InvalidDescription),
        Err(_) => {
            Ok(" ".to_string()) // some files have invalid descriptions
        }
    }
}

fn parse_parameter(
    parameter_blocks: &Vec<u8>,
    index: usize,
    processor: &Processor,
) -> Result<(ParsedParameter, usize), C3dParseError> {
    let mut i = index;
    let num_chars_in_name = parameter_blocks[i] as i8;
    i += 1;
    let group_id = parameter_blocks[i] as i8;
    i += 1;
    let name = parse_parameter_name(&parameter_blocks, i, num_chars_in_name)?.to_ascii_uppercase();
    i += num_chars_in_name.abs() as usize;
    let next_index_bytes = &parameter_blocks[i..i + 2];
    let next_index = processor.u16(next_index_bytes.try_into().unwrap()) as usize + i as usize;
    i += 2;
    let data_type = DataType::try_from(parameter_blocks[i] as i8)?;
    i += 1;
    let num_dimensions = parameter_blocks[i];
    i += 1;
    let dimensions = parse_dimensions(&parameter_blocks, i, num_dimensions, data_type)?;
    i += num_dimensions as usize;
    let (data, data_byte_size) =
        parse_data(&parameter_blocks, i, &dimensions, data_type, processor)?;
    i += data_byte_size;
    let num_chars_in_description = parameter_blocks[i];
    i += 1;
    let description = parse_description(&parameter_blocks, i, num_chars_in_description)?;

    Ok((
        ParsedParameter {
            group_id,
            name,
            data,
            dimensions,
            description,
        },
        next_index,
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
    data_type: DataType,
) -> Result<Vec<u8>, C3dParseError> {
    let mut dimensions = Vec::new();

    for i in 0..num_dimensions {
        dimensions.push(parameter_blocks[index + i as usize]);
    }
    dimensions.retain(|&x| x != 0);
    if data_type != DataType::Char {
        dimensions.retain(|&x| x != 1);
    }
    if dimensions.len() == 0 {
        dimensions.push(1);
    }

    Ok(dimensions)
}

fn parse_data(
    parameter_blocks: &Vec<u8>,
    index: usize,
    dimensions: &Vec<u8>,
    data_type: DataType,
    processor: &Processor,
) -> Result<(ParameterData, usize), C3dParseError> {
    let dimensions_product = &dimensions
        .clone()
        .iter()
        .map(|x| *x as usize)
        .product::<usize>();

    let data_byte_size = dimensions_product * usize::from(data_type);

    if index + data_byte_size > parameter_blocks.len() {
        return Err(C3dParseError::InvalidParameterData);
    }

    let bytes: &[u8] = &parameter_blocks[index..index + data_byte_size];

    Ok((
        ParameterData::new(bytes, dimensions, data_type, processor)?,
        data_byte_size,
    ))
}

#[derive(Debug, Clone)]
struct ParsedGroup {
    id: i8,
    name: String,
    description: String,
}

#[derive(Debug, Clone)]
struct ParsedParameter {
    group_id: i8,
    name: String,
    pub data: ParameterData,
    dimensions: Vec<u8>,
    description: String,
}

const REQUIRED_GROUPS: [&str; 8] = [
    "POINT",
    "ANALOG",
    "FORCE_PLATFORM",
    "EVENT",
    "EVENT_CONTEXT",
    "TRIAL",
    "MANUFACTURER",
    "SEG",
];

fn insert_missing_required_groups(
    groups_map: &mut HashMap<String, (String, HashMap<String, Parameter>)>,
    temp_group_id_to_name: &mut HashMap<i8, String>,
) -> Result<(), C3dParseError> {
    let max_group_id = temp_group_id_to_name.keys().max();
    let mut max_group_id = match max_group_id {
        Some(max_group_id) => *max_group_id,
        None => 0,
    };
    for required_group in &REQUIRED_GROUPS {
        if !groups_map.contains_key(*required_group) {
            max_group_id += 1;
            groups_map.insert(required_group.to_string(), ("".to_string(), HashMap::new()));
            temp_group_id_to_name.insert(max_group_id.clone(), required_group.to_string());
        }
    }
    Ok(())
}
