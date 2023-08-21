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
    /// Parses a C3D file from a file path.
    pub fn load(file_name: &str) -> Result<C3d, C3dParseError> {
        let c3d = C3d::new()?;
        let (c3d, mut file) = c3d.open_file(file_name)?;
        Ok(c3d
            .parse_basic_info(&mut file)?
            .parse_header()?
            .parse_parameters()?
            .parse_data(file)?)
    }

    /// Parses a C3D file from a byte slice.
    pub fn from_bytes(bytes: &[u8]) -> Result<C3d, C3dParseError> {
        Ok(C3d::new()?
            .parse_basic_info_from_bytes(bytes)?
            .parse_header()?
            .parse_parameters()?
            .parse_data_from_bytes(bytes)?)
    }

    /// Parses a C3D file with just the header data.
    pub fn load_header(file_name: &str) -> Result<C3d, C3dParseError> {
        let c3d = C3d::new()?;
        let (c3d, mut file) = c3d.open_file(file_name)?;
        Ok(c3d.parse_basic_info(&mut file)?.parse_header()?)
    }

    /// Parses a C3D file with just the header and parameter data.
    /// The parameter data cannot be parsed without the header data.
    /// The parameter data is parsed into a `Parameters` struct.
    /// The `Parameters` struct can be accessed via the `parameters` field.
    pub fn load_parameters(file_name: &str) -> Result<C3d, C3dParseError> {
        let c3d = C3d::new()?;
        let (c3d, mut file) = c3d.open_file(file_name)?;
        Ok(c3d.parse_basic_info(&mut file)?
            .parse_header()?
            .parse_parameters()?)
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
        let file = File::open(self.file_path.clone().unwrap()).map_err(|e| C3dParseError::ReadError(e))?;
        Ok((self, file))
    }

    fn read_header_bytes(mut self, file: &mut File) -> Result<C3d, C3dParseError> {
        self.bytes.header = [0u8; 512];
        file.read_exact(&mut self.bytes.header)
            .map_err(|e| C3dParseError::ReadError(e))?;
        Ok(self)
    }

    fn read_parameter_bytes(mut self, file: &mut File) -> Result<C3d, C3dParseError> {
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

    fn parse_basic_info(self, file: &mut File) -> Result<C3d, C3dParseError> {
        self.read_header_bytes(file)?.read_parameter_bytes(file)
    }

    fn parse_basic_info_from_bytes(mut self, bytes: &[u8]) -> Result<C3d, C3dParseError> {
        if bytes.len() < 512 {
            return Err(C3dParseError::InsufficientBlocks("header".to_string()));
        }
        self.bytes.header = bytes[0..512].try_into().unwrap();

        self.bytes.parameter_start_block_index = self.bytes.header[0] as usize;

        if bytes.len() < 512 * (self.bytes.parameter_start_block_index) {
            return Err(C3dParseError::InsufficientBlocks("parameter".to_string()));
        }
        let parameter_start_block = &bytes[512..1024];

        self.processor =
            Processor::from_parameter_start_block(parameter_start_block.try_into().unwrap())?;
        self.bytes.data_start_block_index =
            self.processor
                .u16([self.bytes.header[16], self.bytes.header[17]]) as usize;

        if bytes.len() < 512 * (self.bytes.data_start_block_index) {
            return Err(C3dParseError::InsufficientBlocks("data".to_string()));
        }

        self.bytes.parameter = bytes[512..(512 * (self.bytes.data_start_block_index - 1))]
            .try_into()
            .unwrap();

        Ok(self)
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

    fn parse_data(self, file: File) -> Result<C3d, C3dParseError> {
        self.read_data_bytes(file)?.parse_data_bytes()
    }

    fn parse_data_from_bytes(mut self, bytes: &[u8]) -> Result<C3d, C3dParseError> {
        let data_start_byte = 512 * (self.bytes.data_start_block_index - 1);
        if bytes.len() < data_start_byte {
            return Err(C3dParseError::InsufficientBlocks("data".to_string()));
        }
        self.bytes.data = bytes[data_start_byte..].to_vec();
        self.parse_data_bytes()
    }

    fn parse_data_bytes(mut self) -> Result<C3d, C3dParseError> {
        self.data.point_frames = match self.parameters.get_data("POINT", "FRAMES") {
            Some(ParameterData::Integer(frames, _)) => {
                frames[0] as u16 as usize
            }
            Some(ParameterData::Float(frames, _)) => {
                frames[0] as u16 as usize
            }
            _ => 0,
        };

        self.data
            .parse(&self.bytes.data, &self.parameters, &self.processor)?;
        Ok(self)
    }
}

/// The steps in the process of loading and parsing a C3D file.
/// This is used in conjunction with the `test_load_file` function
/// to determine where a file failed to load or parse.
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

/// A rudimentary test to check that a file can be loaded and parsed.
/// This is not a comprehensive test, but it is a good first check.
/// It is not intended to be used as a unit test.
pub fn test_load_file(file_name: &str) -> ProcessStep {
    let c3d = match C3d::new() {
        Ok(c3d) => c3d,
        Err(_) => return ProcessStep::MakeEmpty,
    };
    let (c3d, mut file) = match c3d.open_file(file_name) {
        Ok(c3d) => c3d,
        Err(_) => return ProcessStep::LoadFile,
    };
    let c3d = match c3d.parse_basic_info(&mut file) {
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
