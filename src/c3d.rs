use crate::parameters::{ParameterData, Parameters};

use crate::bytes::Bytes;
use crate::data::Data;
use crate::events::Events;
use crate::processor::Processor;
use crate::{C3d, C3dParseError};

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;


impl PartialEq for C3d {
    fn eq(&self, other: &Self) -> bool {
        self.parameters == other.parameters
            && self.data == other.data
            && self.events == other.events
    }
}

impl C3d {
    pub fn load(file_name: &str) -> Result<C3d, C3dParseError> {
        let c3d = C3d::new()?;
        let (c3d, file) = 
            c3d.open_file(file_name)?;
            c3d.parse_basic_info(file)?
            .parse_header()?
            .parse_parameters()?
            .parse_data(file)?;

        Ok(c3d)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<C3d, C3dParseError> {
        let c3d = C3d::new()?
            .parse_basic_info_from_bytes(bytes)?
            .parse_header()?
            .parse_parameters()?
            .parse_data_from_bytes(bytes)?;

        Ok(c3d)
    }

    pub fn load_header(file_name: &str) -> Result<C3d, C3dParseError> {
        let c3d = C3d::new()?;
        let (c3d, file) = 
            c3d.open_file(file_name)?;
            c3d.parse_basic_info(file)?
            .parse_header()?;

        Ok(c3d)
    }

    pub fn load_parameters(file_name: &str) -> Result<C3d, C3dParseError> {
        let c3d = C3d::new()?;
        let (c3d, file) = 
            c3d.open_file(file_name)?;
            c3d.parse_basic_info(file)?
            .parse_header()?
            .parse_parameters()?;

        Ok(c3d)
    }

    pub fn new() -> Result<C3d, C3dParseError> {
        Ok(C3d {
            file_path: None,
            bytes: Bytes::new(),
            parameters: Parameters::new(),
            processor: Processor::new(),
            data: Data::new(),
            events: Events::new(),
        })
    }

    fn open_file(mut self, file_name: &str) -> Result<(C3d, File), C3dParseError> {
        self.file_path = Some(PathBuf::from(file_name));
        let file = File::open(&self.file_path.unwrap()).map_err(|e| C3dParseError::ReadError(e))?;
        Ok((self, file))
    }

    fn read_header_bytes(mut self, mut file: File) -> Result<C3d, C3dParseError> {
        self.bytes.header = [0u8; 512];
        file.read_exact(&mut self.bytes.header)
                .map_err(|e| C3dParseError::ReadError(e))?;
        Ok(self)
    }

    fn read_parameter_bytes(mut self, mut file: File) -> Result<C3d, C3dParseError> {
        self.bytes.parameter_start_block_index = self.bytes.header[0] as usize;

        let blocks_to_skip = self.bytes.parameter_start_block_index - 2;
        file.seek(SeekFrom::Current((512 * blocks_to_skip) as i64))
            .map_err(|e| C3dParseError::ReadError(e))?;

        let mut parameter_start_block = [0u8; 512];
        file.read_exact(&mut parameter_start_block)
            .map_err(|e| C3dParseError::ReadError(e))?;

        self.processor = Processor::from_parameter_start_block(parameter_start_block)?;
        self.bytes.data_start_block_index =
            self.processor
                .u16([self.bytes.header[16], self.bytes.header[17]]) as usize;

        let mut parameter_bytes_tail = Vec::with_capacity(
            (self.bytes.data_start_block_index - self.bytes.parameter_start_block_index - 1) * 512,
        ) as Vec<u8>;

        for _ in 0..(self.bytes.data_start_block_index - self.bytes.parameter_start_block_index - 1)
        {
            let mut block = [0u8; 512];
            file.read_exact(&mut block)
                .map_err(|e| C3dParseError::ReadError(e))?;
            parameter_bytes_tail.extend(block.iter());
        }

        self.bytes.parameter = [
            parameter_start_block.as_slice(),
            parameter_bytes_tail.as_slice(),
        ]
        .concat();

        Ok(self)
    }

    fn parse_basic_info(self, mut file: File) -> Result<C3d, C3dParseError> {
        self.read_header_bytes(file)?.read_parameter_bytes(file)
    }

    fn parse_header(mut self) -> Result<C3d, C3dParseError> {
        self.data.points_per_frame = self
            .processor
            .u16(self.bytes.header[2..4].try_into().unwrap());
        self.data.analog_samples_per_frame = self
            .processor
            .u16(self.bytes.header[4..6].try_into().unwrap());
        self.data.first_frame = self
            .processor
            .u16(self.bytes.header[6..8].try_into().unwrap());
        self.data.last_frame = self
            .processor
            .u16(self.bytes.header[8..10].try_into().unwrap());
        self.data.max_interpolation_gap = self
            .processor
            .u16(self.bytes.header[10..12].try_into().unwrap());
        self.data.scale_factor = self
            .processor
            .f32(self.bytes.header[12..16].try_into().unwrap());
        self.data.analog_channels = self
            .processor
            .u16(self.bytes.header[18..20].try_into().unwrap());
        self.data.frame_rate = self
            .processor
            .f32(self.bytes.header[20..24].try_into().unwrap());
        self.events = Events::from_header_block(&self.bytes.header, &self.processor)?;
        Ok(self)
    }

    fn parse_parameters(mut self) -> Result<C3d, C3dParseError> {
        self.parameters =
            Parameters::parse_parameter_blocks(&self.bytes.parameter, &self.processor)?;
        Ok(self)
    }

    fn read_data_bytes(mut self, mut file: File) -> Result<C3d, C3dParseError> {
        self.bytes.data = Vec::new();

        file.read_to_end(&mut self.bytes.data)
            .map_err(|e| C3dParseError::ReadError(e))?;
        Ok(self)
    }

    fn parse_data(self, mut file: File) -> Result<C3d, C3dParseError> {
        self.read_data_bytes(file)?.parse_data_bytes()
    }

    fn parse_data_bytes(mut self) -> Result<C3d, C3dParseError> {
        self.data.point_frames = match self.parameters.get_data("POINT", "FRAMES") {
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
        let start_field_parameter = self.parameters.get_int_vec("TRIAL", "ACTUAL_START_FIELD");
        let end_field_parameter = self.parameters.get_int_vec("TRIAL", "ACTUAL_END_FIELD");

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
        self.data.parse(&self.bytes.data, &self.parameters, &self.processor)?;
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
    let (c3d, file) = match c3d.open_file(file_name) {
        Ok(c3d) => c3d,
        Err(_) => return ProcessStep::LoadFile,
    };
    let c3d = match c3d.parse_basic_info(file) {
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
    match c3d.parse_data(file) {
        Ok(c3d) => c3d,
        Err(_) => return ProcessStep::ParseData,
    };
    ProcessStep::Complete
}
