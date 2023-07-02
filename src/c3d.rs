use crate::parameters::{parse_parameter_blocks, ParameterData};

use crate::data::Data;
use crate::events::Events;
use crate::processor::Processor;
use crate::{C3d, C3dParseError};

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

impl PartialEq for C3d {
    fn eq(&self, other: &Self) -> bool {
        self.group_descriptions == other.group_descriptions
//            &&  self.parameters == other.parameters
            && self.data == other.data
            && self.events == other.events
    }
}

impl C3d {
    pub fn load(file_name: &str) -> Result<C3d, C3dParseError> {
        let c3d = C3d::new()?
            .open_file(file_name)?
            .parse_basic_info()?
            .parse_header()?
            .parse_parameters()?
            .parse_data()?;

        Ok(c3d)
    }

    pub fn load_header(file_name: &str) -> Result<C3d, C3dParseError> {
        let c3d = C3d::new()?
            .open_file(file_name)?
            .parse_basic_info()?
            .parse_header()?;

        Ok(c3d)
    }

    pub fn load_parameters(file_name: &str) -> Result<C3d, C3dParseError> {
        let c3d = C3d::new()?
            .open_file(file_name)?
            .parse_basic_info()?
            .parse_header()?
            .parse_parameters()?;

        Ok(c3d)
    }

    //    pub fn get_parameter(
    //        &self,
    //        group_name: &str,
    //        parameter_name: &str,
    //    ) -> Option<(&ParameterData, &String)> {
    //        let group = self.parameters.get(group_name)?;
    //        let parameter = group.get(parameter_name)?;
    //        Some((&parameter.0, &parameter.1))
    //    }

    pub fn get_parameter_data(
        &self,
        group_name: &str,
        parameter_name: &str,
    ) -> Option<&ParameterData> {
        let group = self.parameters.get(group_name)?;
        let parameter = group.get(parameter_name)?;
        Some(&parameter.0)
    }

    pub fn get_parameter_float(&self, group_name: &str, parameter_name: &str) -> Option<f32> {
        let group = self.parameters.get(group_name)?;
        let parameter = group.get(parameter_name)?;
        match &parameter.0 {
            ParameterData::Float(data) => {
                if data.len() == 1 {
                    data.first().cloned()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn get_parameter_float_vec(
        &self,
        group_name: &str,
        parameter_name: &str,
    ) -> Option<Vec<f32>> {
        let group = self.parameters.get(group_name)?;
        let parameter = group.get(parameter_name)?;
        match &parameter.0 {
            ParameterData::Float(data) => {
                if data.ndim() == 1 {
                    Some(data.to_owned().into_raw_vec())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn get_parameter_int(&self, group_name: &str, parameter_name: &str) -> Option<i16> {
        let group = self.parameters.get(group_name)?;
        let parameter = group.get(parameter_name)?;
        match &parameter.0 {
            ParameterData::Integer(data) => {
                if data.len() == 1 {
                    data.first().cloned()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn get_parameter_int_vec(
        &self,
        group_name: &str,
        parameter_name: &str,
    ) -> Option<Vec<i16>> {
        let group = self.parameters.get(group_name)?;
        let parameter = group.get(parameter_name)?;
        match &parameter.0 {
            ParameterData::Integer(data) => {
                if data.ndim() == 1 {
                    Some(data.to_owned().into_raw_vec())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn get_parameter_string(&self, group_name: &str, parameter_name: &str) -> Option<String> {
        let group = self.parameters.get(group_name)?;
        let parameter = group.get(parameter_name)?;
        match &parameter.0 {
            ParameterData::Char(data) => {
                if data.ndim() == 1 {
                    let mut string = String::new();
                    for c in data {
                        string.push(*c as char);
                    }
                    Some(string)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn get_parameter_string_vec(
        &self,
        group_name: &str,
        parameter_name: &str,
    ) -> Option<Vec<String>> {
        let group = self.parameters.get(group_name)?;
        let parameter = group.get(parameter_name)?;
        match &parameter.0 {
            ParameterData::Char(data) => {
                if data.ndim() == 2 {
                    let mut strings = Vec::new();
                    for column in data.columns() {
                        let mut string = String::new();
                        for c in column {
                            string.push(*c as char);
                        }
                        strings.push(string);
                    }
                    Some(strings)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn new() -> Result<C3d, C3dParseError> {
        let path = PathBuf::from("");
        Ok(C3d {
            file_path: path,
            file: None,
            header_bytes: [0; 512],
            parameter_bytes: Vec::new(),
            parameters: HashMap::new(),
            group_descriptions: HashMap::new(),
            parameter_start_block_index: 0,
            data_start_block_index: 0,
            data_bytes: Vec::new(),
            processor: Processor::new(),
            data: Data::new(),
            events: Events::new(),
        })
    }

    fn open_file(mut self, file_name: &str) -> Result<C3d, C3dParseError> {
        self.file_path = PathBuf::from(file_name);
        self.file = Some(File::open(&self.file_path).map_err(|e| C3dParseError::ReadError(e))?);
        Ok(self)
    }

    fn read_header_bytes(mut self) -> Result<C3d, C3dParseError> {
        self.header_bytes = [0u8; 512];

        if self.file.is_none() {
            return Err(C3dParseError::FileNotOpen);
        }

        let file = self.file.as_mut().unwrap();
        file.read_exact(&mut self.header_bytes)
            .map_err(|e| C3dParseError::ReadError(e))?;
        Ok(self)
    }

    fn read_parameter_bytes(mut self) -> Result<C3d, C3dParseError> {
        self.parameter_start_block_index = self.header_bytes[0] as usize;

        if self.file.is_none() {
            return Err(C3dParseError::FileNotOpen);
        }

        let file = self.file.as_mut().unwrap();
        let blocks_to_skip = self.parameter_start_block_index - 2;
        file.seek(SeekFrom::Current((512 * blocks_to_skip) as i64))
            .map_err(|e| C3dParseError::ReadError(e))?;

        let mut parameter_start_block = [0u8; 512];
        file.read_exact(&mut parameter_start_block)
            .map_err(|e| C3dParseError::ReadError(e))?;

        self.processor = Processor::from_parameter_start_block(parameter_start_block)?;
        self.data_start_block_index =
            self.processor
                .u16([self.header_bytes[16], self.header_bytes[17]]) as usize;

        let mut parameter_bytes_tail = Vec::with_capacity(
            (self.data_start_block_index - self.parameter_start_block_index - 1) * 512,
        ) as Vec<u8>;

        for _ in 0..(self.data_start_block_index - self.parameter_start_block_index - 1) {
            let mut block = [0u8; 512];
            file.read_exact(&mut block)
                .map_err(|e| C3dParseError::ReadError(e))?;
            parameter_bytes_tail.extend(block.iter());
        }

        self.parameter_bytes = [
            parameter_start_block.as_slice(),
            parameter_bytes_tail.as_slice(),
        ]
        .concat();

        Ok(self)
    }

    fn parse_basic_info(self) -> Result<C3d, C3dParseError> {
        self.read_header_bytes()?.read_parameter_bytes()
    }

    fn parse_header(mut self) -> Result<C3d, C3dParseError> {
        self.data.points_per_frame = self
            .processor
            .u16(self.header_bytes[2..4].try_into().unwrap());
        self.data.analog_samples_per_frame = self
            .processor
            .u16(self.header_bytes[4..6].try_into().unwrap());
        self.data.first_frame = self
            .processor
            .u16(self.header_bytes[6..8].try_into().unwrap());
        self.data.last_frame = self
            .processor
            .u16(self.header_bytes[8..10].try_into().unwrap());
        self.data.max_interpolation_gap = self
            .processor
            .u16(self.header_bytes[10..12].try_into().unwrap());
        self.data.scale_factor = self
            .processor
            .f32(self.header_bytes[12..16].try_into().unwrap());
        self.data.analog_channels = self
            .processor
            .u16(self.header_bytes[18..20].try_into().unwrap());
        self.data.frame_rate = self
            .processor
            .f32(self.header_bytes[20..24].try_into().unwrap());
        self.events = Events::from_header_block(&self.header_bytes, &self.processor)?;
        Ok(self)
    }

    fn parse_parameters(mut self) -> Result<C3d, C3dParseError> {
        let (parameters, group_descriptions) =
            parse_parameter_blocks(&self.parameter_bytes, &self.processor)?;

        self.parameters = parameters;
        self.group_descriptions = group_descriptions;

        Ok(self)
    }

    fn read_data_bytes(mut self) -> Result<C3d, C3dParseError> {
        self.data_bytes = Vec::new();

        if self.file.is_none() {
            return Err(C3dParseError::FileNotOpen);
        }

        let file = self.file.as_mut().unwrap();
        file.read_to_end(&mut self.data_bytes)
            .map_err(|e| C3dParseError::ReadError(e))?;
        Ok(self)
    }

    fn parse_data(self) -> Result<C3d, C3dParseError> {
        self.read_data_bytes()?.parse_data_bytes()
    }

    fn parse_data_bytes(mut self) -> Result<C3d, C3dParseError> {
        self.data.point_frames = match self.get_parameter_data("POINT", "FRAMES") {
            Some(ParameterData::Integer(frames)) => {
                let frames = frames.first();
                if let Some(&frames) = frames {
                    frames as u16 as usize
                } else {
                    0
                }
            }
            Some(ParameterData::Float(frames)) => {
                let frames = frames.first();
                if let Some(&frames) = frames {
                    frames as u16 as usize
                } else {
                    0
                }
            }
            _ => 0,
        };
        let start_field_parameter = self.get_parameter_int_vec("TRIAL", "ACTUAL_START_FIELD");
        let end_field_parameter = self.get_parameter_int_vec("TRIAL", "ACTUAL_END_FIELD");

        if start_field_parameter.is_some() && end_field_parameter.is_some() {
            let start_field_parameter = start_field_parameter.unwrap();
            let end_field_parameter = end_field_parameter.unwrap();

            if start_field_parameter.len() == 2 && end_field_parameter.len() == 2 {
                let start_one = start_field_parameter.first().unwrap();
                let start_two = start_field_parameter.last().unwrap();
                let end_one = end_field_parameter.first().unwrap();
                let end_two = end_field_parameter.last().unwrap();

                self.data.trial_start_frame =
                    *start_one as u16 as usize + (*start_two as u16 * 65535) as usize;
                self.data.trial_end_frame =
                    *end_one as u16 as usize + (*end_two as u16 * 65535) as usize;
                //self.data.num_frames = end_field - start_field + 1;
            }
        }
        self.data.parse(&self.data_bytes, &self.processor)?;
        Ok(self)
    }
}

#[derive(Debug, PartialEq)]
pub enum ProcessStep {
    MakeEmpty,
    LoadFile,
    ParseBasicInfo,
    ParseHeader,
    ParseParameters,
    ParseData,
    Complete,
}

pub fn test_load_file(file_name: &str) -> ProcessStep {
    let c3d = match C3d::new() {
        Ok(c3d) => c3d,
        Err(_) => return ProcessStep::MakeEmpty,
    };
    let c3d = match c3d.open_file(file_name) {
        Ok(c3d) => c3d,
        Err(_) => return ProcessStep::LoadFile,
    };
    let c3d = match c3d.parse_basic_info() {
        Ok(c3d) => c3d,
        Err(_) => return ProcessStep::ParseBasicInfo,
    };
    let c3d = match c3d.parse_header() {
        Ok(c3d) => c3d,
        Err(_) => return ProcessStep::ParseHeader,
    };
    let c3d = match c3d.parse_parameters() {
        Ok(c3d) => c3d,
        Err(_) => return ProcessStep::ParseParameters,
    };
    match c3d.parse_data() {
        Ok(c3d) => c3d,
        Err(_) => return ProcessStep::ParseData,
    };
    ProcessStep::Complete
}
