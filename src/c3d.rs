//! Includes the C3d struct implementation and high-level functions for reading and writing C3D files.
use crate::analog::Analog;
use crate::data::DataFormat;
use crate::forces::ForcePlatforms;
use crate::manufacturer::Manufacturer;
use crate::parameters::Parameters;
use crate::points::Points;
use crate::seg::Seg;

use crate::events::Events;
use crate::processor::Processor;
use crate::{C3dParseError, C3dWriteError};

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use std::fmt::{Debug, Formatter};

/// Represents a parsed C3D file.
/// Each field contains the data from the corresponding section of the file.
pub struct C3d {
    pub parameters: Parameters,
    processor: Processor,
    pub points: Points,
    pub analog: Analog,
    pub events: Events,
    pub manufacturer: Manufacturer,
    pub seg: Seg,
    pub forces: ForcePlatforms,
    header_bytes: [u8; 512],
}

impl PartialEq for C3d {
    fn eq(&self, other: &Self) -> bool {
        //        self.processor == other.processor
        self.points == other.points
            && self.analog == other.analog
            && self.manufacturer == other.manufacturer
            && self.seg == other.seg
            && self.forces == other.forces
            && self.parameters == other.parameters
            && self.events == other.events
    }
}

impl Debug for C3d {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("C3d")
            .field("processor", &self.processor)
            .field("points", &self.points)
            .field("analog", &self.analog)
            .field("manufacturer", &self.manufacturer)
            .field("seg", &self.seg)
            .field("forces", &self.forces)
            .field("parameters", &self.parameters)
            .field("events", &self.events)
            .finish()
    }
}

impl Default for C3d {
    fn default() -> Self {
        C3d {
            processor: Processor::default(),
            points: Points::default(),
            analog: Analog::default(),
            parameters: Parameters::default(),
            events: Events::default(),
            manufacturer: Manufacturer::default(),
            seg: Seg::default(),
            forces: ForcePlatforms::default(),
            header_bytes: [0u8; 512],
        }
    }
}

impl ToString for C3d {
    fn to_string(&self) -> String {
        let mut s = String::new();
        s.push_str(self.processor.to_string().as_str());
        s.push_str(self.points.to_string().as_str());
        s.push_str(self.analog.to_string().as_str());
        s.push_str(self.forces.to_string().as_str());
        s.push_str(self.parameters.to_string().as_str());
        s.push_str(self.events.to_string().as_str());
        s.push_str(self.manufacturer.to_string().as_str());
        s.push_str(self.seg.to_string().as_str());
        s
    }
}

impl C3d {
    /// Parses a C3D file from a file path string.
    /// loading from a string is less inclusive than loading from a PathBuf
    /// <https://users.rust-lang.org/t/pathbuf-and-path-why-not-string/28777>
    pub fn load(file_name: &str) -> Result<C3d, C3dParseError> {
        C3d::load_path(PathBuf::from(file_name))
    }

    /// Parses a C3D file from a file path.
    /// PathBuf is more inclusive than String
    /// <https://users.rust-lang.org/t/pathbuf-and-path-why-not-string/28777>
    pub fn load_path(file_path: PathBuf) -> Result<C3d, C3dParseError> {
        let c3d = C3d::new();
        let (c3d, mut file) = c3d.open_file(file_path)?;
        let (c3d, header_bytes, parameter_bytes, _) = c3d.parse_basic_info(&mut file)?;
        Ok(c3d
            .parse_header(&header_bytes)?
            .parse_parameters(&header_bytes, &parameter_bytes)?
            .parse_data(file)?)
    }

    /// Parses a C3D file from a byte slice.
    pub fn from_bytes(bytes: &[u8]) -> Result<C3d, C3dParseError> {
        let (c3d, header_bytes, parameter_bytes, data_start_block_index) =
            C3d::new().parse_basic_info_from_bytes(bytes)?;
        Ok(c3d
            .parse_header(&header_bytes)?
            .parse_parameters(&header_bytes, &parameter_bytes)?
            .parse_data_from_bytes(bytes, data_start_block_index)?)
    }

    /// Parses a C3D file with just the header data.
    pub fn load_header(file_name: PathBuf) -> Result<C3d, C3dParseError> {
        let c3d = C3d::new();
        let (c3d, mut file) = c3d.open_file(file_name)?;
        let (c3d, header_bytes, _, _) = c3d.parse_basic_info(&mut file)?;
        Ok(c3d.parse_header(&header_bytes)?)
    }

    /// Parses a C3D file with just the header and parameter data.
    /// The parameter data cannot be parsed without the header data.
    /// The parameter data is parsed into a `Parameters` struct.
    /// The `Parameters` struct can be accessed via the `parameters` field.
    pub fn load_parameters(file_name: PathBuf) -> Result<C3d, C3dParseError> {
        let c3d = C3d::new();
        let (c3d, mut file) = c3d.open_file(file_name)?;
        let (c3d, header_bytes, parameter_bytes, _) = c3d.parse_basic_info(&mut file)?;
        Ok(c3d
            .parse_header(&header_bytes)?
            .parse_parameters(&header_bytes, &parameter_bytes)?)
    }

    pub fn new() -> C3d {
        C3d::default()
    }

    fn force_analog_data(&self, force_plate: usize, frame: usize) -> Option<[f32; 8]> {
        if self.forces.len() <= force_plate {
            return None;
        }
        let channels = self.forces[force_plate].channels;
        let mut analog = [0f32; 8];
        for i in 0..8 {
            let channel_index = channels[i];
            if channel_index == 0 {
                analog[i] = 0.0;
                continue;
            }
            if self.analog.cols() <= (channel_index - 1) as usize
                || self.analog.rows() <= frame * self.analog.samples_per_channel_per_frame as usize
            {
                return None;
            }
            analog[i] = self.analog[frame * self.analog.samples_per_channel_per_frame as usize]
                [(channel_index - 1) as usize] as f32;
        }
        Some(analog)
    }

    pub fn force(&self, force_plate: usize, frame: usize) -> Option<[f32; 3]> {
        let analog = self.force_analog_data(force_plate, frame)?;
        self.forces.force_from_analog(analog, force_plate)
    }

    pub fn center_of_pressure(&self, force_plate: usize, frame: usize) -> Option<[f32; 2]> {
        let analog = self.force_analog_data(force_plate, frame)?;
        self.forces
            .center_of_pressure_from_analog(analog, force_plate)
    }

    fn open_file(self, file_path: PathBuf) -> Result<(C3d, File), C3dParseError> {
        let file = File::open(file_path).map_err(|e| C3dParseError::ReadError(e))?;
        Ok((self, file))
    }

    fn parse_basic_info(
        mut self,
        file: &mut File,
    ) -> Result<(C3d, [u8; 512], Vec<u8>, usize), C3dParseError> {
        let header_bytes = read_header_bytes(file)?;
        let (processor, parameter_bytes, data_start_block_index) =
            read_parameter_bytes(file, &header_bytes)?;
        self.processor = processor;
        Ok((self, header_bytes, parameter_bytes, data_start_block_index))
    }

    fn parse_basic_info_from_bytes(
        mut self,
        bytes: &[u8],
    ) -> Result<(C3d, [u8; 512], Vec<u8>, usize), C3dParseError> {
        if bytes.len() < 512 {
            return Err(C3dParseError::InsufficientBlocks("header".to_string()));
        }
        let header_bytes: [u8; 512] = bytes[0..512].try_into().unwrap();

        let parameter_start_block_index = header_bytes[0] as usize;

        if bytes.len() < 512 * (parameter_start_block_index) {
            return Err(C3dParseError::InsufficientBlocks("parameter".to_string()));
        }
        let blocks_to_skip = 512 * (parameter_start_block_index - 1);
        let parameter_start_block: [u8; 512] = bytes[blocks_to_skip..(blocks_to_skip + 512)]
            .try_into()
            .unwrap();

        self.processor =
            Processor::from_parameter_start_block(parameter_start_block.try_into().unwrap())?;
        let data_start_block_index =
            self.processor.u16([header_bytes[16], header_bytes[17]]) as usize;

        if bytes.len() < 512 * (data_start_block_index) {
            return Err(C3dParseError::InsufficientBlocks("data".to_string()));
        }

        let parameter_bytes: Vec<u8> = bytes[512..(512 * (data_start_block_index - 1))]
            .try_into()
            .unwrap();

        Ok((self, header_bytes, parameter_bytes, data_start_block_index))
    }

    fn parse_header(mut self, header_bytes: &[u8; 512]) -> Result<C3d, C3dParseError> {
        self.points = Points::parse_header(&header_bytes, &self.processor);
        self.analog = Analog::parse_header(&header_bytes, &self.processor);
        self.header_bytes = header_bytes.clone();
        Ok(self)
    }

    fn parse_parameters(
        mut self,
        header_bytes: &[u8; 512],
        parameter_bytes: &Vec<u8>,
    ) -> Result<C3d, C3dParseError> {
        self.parameters = Parameters::parse_parameter_blocks(parameter_bytes, &self.processor)?;
        self.events = Events::from_header_and_parameters(
            &header_bytes,
            &mut self.parameters,
            &self.processor,
        )?;
        self.manufacturer = Manufacturer::from_parameters(&mut self.parameters)?;
        self.seg = Seg::from_parameters(&mut self.parameters)?;
        self.forces = ForcePlatforms::from_parameters(&mut self.parameters)?;
        Ok(self)
    }

    fn parse_data(self, file: File) -> Result<C3d, C3dParseError> {
        let data_bytes = read_data_bytes(file)?;
        self.parse_data_bytes(data_bytes)
    }

    fn parse_data_from_bytes(
        self,
        bytes: &[u8],
        data_start_block_index: usize,
    ) -> Result<C3d, C3dParseError> {
        let data_start_byte = 512 * (data_start_block_index - 1);
        if bytes.len() < data_start_byte {
            return Err(C3dParseError::InsufficientBlocks("data".to_string()));
        }
        self.parse_data_bytes(bytes[data_start_byte..].to_vec())
    }

    fn parse_data_bytes(mut self, data_bytes: Vec<u8>) -> Result<C3d, C3dParseError> {
        let (_, num_frames) = self.points.parse(
            &data_bytes,
            &mut self.parameters,
            &self.processor,
            self.analog.samples_per_frame,
        )?;
        self.analog.parse(
            &data_bytes,
            &mut self.parameters,
            &self.processor,
            num_frames,
            &self.points.format,
            self.points.cols(),
        )?;
        Ok(self)
    }

    /// A function to write a C3D header to bytes.
    fn write_header(&self, data_start_block_index: u16) -> Result<[u8; 512], C3dWriteError> {
        let mut header_bytes = [0u8; 512];
        header_bytes[0] = 2;
        header_bytes[1] = 80;
        let temp = self.processor.u16_to_bytes(self.points.cols() as u16);
        header_bytes[2] = temp[0];
        header_bytes[3] = temp[1];
        let temp = self
            .processor
            .u16_to_bytes(self.analog.samples_per_frame as u16);
        header_bytes[4] = temp[0];
        header_bytes[5] = temp[1];
        let temp = self.processor.u16_to_bytes(self.points.first_frame);
        header_bytes[6] = temp[0];
        header_bytes[7] = temp[1];
        let temp = self.processor.u16_to_bytes(self.points.last_frame);
        header_bytes[8] = temp[0];
        header_bytes[9] = temp[1];
        let temp = self
            .processor
            .u16_to_bytes(self.points.max_interpolation_gap);
        header_bytes[10] = temp[0];
        header_bytes[11] = temp[1];
        let temp = match self.points.format {
            DataFormat::Float => self.processor.f32_to_bytes(-self.points.scale_factor),
            DataFormat::Integer => self.processor.f32_to_bytes(self.points.scale_factor),
        };
        header_bytes[12] = temp[0];
        header_bytes[13] = temp[1];
        header_bytes[14] = temp[2];
        header_bytes[15] = temp[3];
        let temp = self.processor.u16_to_bytes(data_start_block_index);
        header_bytes[16] = temp[0];
        header_bytes[17] = temp[1];
        let temp = self
            .processor
            .u16_to_bytes(self.analog.samples_per_channel_per_frame);
        header_bytes[18] = temp[0];
        header_bytes[19] = temp[1];
        let temp = self.processor.f32_to_bytes(self.points.frame_rate as f32);
        header_bytes[20] = temp[0];
        header_bytes[21] = temp[1];
        header_bytes[22] = temp[2];
        header_bytes[23] = temp[3];
        // words 13 to 149 are reserved for future use and should be set to the previous
        // self.header_bytes value
        for i in 0..137 {
            header_bytes[24 + i * 2] = self.header_bytes[24 + i * 2];
            header_bytes[25 + i * 2] = self.header_bytes[25 + i * 2];
        }
        let temp = match self.events.supports_events_labels {
            true => self.processor.u16_to_bytes(12345),
            false => self.processor.u16_to_bytes(0),
        };
        match self.events.supports_events_labels {
            true => {
                header_bytes[298] = temp[0];
                header_bytes[299] = temp[1];
                let temp = self.processor.u16_to_bytes(self.events.len() as u16);
                header_bytes[300] = temp[0];
                header_bytes[301] = temp[1];

                for i in 0..self.events.len() {
                    let temp = self.processor.f32_to_bytes(self.events[i].time);
                    header_bytes[304 + i * 4] = temp[0];
                    header_bytes[305 + i * 4] = temp[1];
                    header_bytes[306 + i * 4] = temp[2];
                    header_bytes[307 + i * 4] = temp[3];
                    let temp = match self.events[i].display_flag {
                        true => 0,
                        false => 1,
                    };
                    header_bytes[376 + i] = temp;
                    let temp = self.events[i].id;
                    header_bytes[396 + i * 4] = temp[0] as u8;
                    header_bytes[397 + i * 4] = temp[1] as u8;
                    header_bytes[398 + i * 4] = temp[2] as u8;
                    header_bytes[399 + i * 4] = temp[3] as u8;
                }
            }
            false => {
                header_bytes[298] = self.header_bytes[298];
                header_bytes[299] = self.header_bytes[299];
            }
        }

        Ok(header_bytes)
    }

    fn write_parameter_blocks(&self) -> Result<Vec<u8>, C3dWriteError> {
        let mut parameter_bytes: Vec<u8> = Vec::new();
        parameter_bytes.append(vec![0, 0, 0].as_mut());
        parameter_bytes.push(match self.processor {
            Processor::Intel => 0x54,
            Processor::Dec => 0x55,
            Processor::SgiMips => 0x56,
        });
        let (group_bytes, group_names_to_ids) = self.parameters.write_groups(&self.processor)?;
        parameter_bytes.extend(group_bytes);

        let num_frames = match self.points.rows() == 0
            && self.analog.rows() > 0
            && self.analog.samples_per_channel_per_frame != 0
        {
            true => self.analog.rows() / self.analog.samples_per_channel_per_frame as usize,
            false => self.points.rows(),
        };
        parameter_bytes.extend(self.points.write_parameters(
            &self.processor,
            &group_names_to_ids,
            num_frames,
        )?);
        parameter_bytes.extend(
            self.analog
                .write_parameters(&self.processor, &group_names_to_ids)?,
        );
        parameter_bytes.extend(self.forces.write(&self.processor, &group_names_to_ids)?);
        parameter_bytes.extend(self.events.write(&self.processor, &group_names_to_ids)?);
        parameter_bytes.extend(
            self.manufacturer
                .write(&self.processor, &group_names_to_ids)?,
        );
        parameter_bytes.extend(self.seg.write(&self.processor, &group_names_to_ids)?);
        parameter_bytes.extend(
            self.parameters
                .write_parameters(&self.processor, &group_names_to_ids)?,
        );

        let num_blocks = parameter_bytes.len() / 512 + 1;
        parameter_bytes[2] = num_blocks as u8;
        Ok(parameter_bytes)
    }

    fn write_data(&self) -> Result<Vec<u8>, C3dWriteError> {
        let mut data_bytes = Vec::new();
        let num_frames = match self.points.rows() == 0
            && self.analog.rows() > 0
            && self.analog.samples_per_channel_per_frame != 0
        {
            true => self.analog.rows() / self.analog.samples_per_channel_per_frame as usize,
            false => self.points.rows(),
        };
        for i in 0..num_frames {
            data_bytes.extend(self.points.write_frame(i, &self.processor));
            data_bytes.extend(
                self.analog
                    .write_frame(i, &self.processor, &self.points.format),
            );
        }
        Ok(data_bytes)
    }

    pub fn write(&self, file_name: &str) -> Result<&Self, C3dWriteError> {
        self.write_path(PathBuf::from(file_name))
    }

    /// A function to write a C3D file to a file path.
    /// This function will overwrite any existing file.
    /// If the file path does not exist, it will be created.
    /// If the file path is a directory, an error will be returned.
    /// If the file path is not writable, an error will be returned.
    /// If the file path is not a valid UTF-8 string, an error will be returned.
    pub fn write_path(&self, file_name: PathBuf) -> Result<&Self, C3dWriteError> {
        // Check if the file path is a directory.
        if file_name.is_dir() {
            return Err(C3dWriteError::InvalidFilePath(file_name));
        }
        // Check if file_name ends with ".c3d", ".C3D", ".c3D", or ".C3d".
        let extension = file_name
            .extension()
            .unwrap()
            .to_string_lossy()
            .to_lowercase();
        if !extension.eq("c3d") {
            return Err(C3dWriteError::InvalidFileExtension(
                file_name.to_string_lossy().to_string(),
            ));
        }
        let mut file = File::create(file_name.clone())
            .map_err(|e| C3dWriteError::WriteError(file_name.clone(), e))?;
        let mut parameter_bytes = self.write_parameter_blocks()?;
        if parameter_bytes.len() % 512 != 0 {
            // add padding
            let padding = 512 - parameter_bytes.len() % 512;
            parameter_bytes.extend(vec![0u8; padding]);
        }
        let data_start_block_index = 2 + parameter_bytes.len() / 512;
        let header_bytes = self.write_header(data_start_block_index as u16)?;
        let data_bytes = self.write_data()?;

        file.write_all(&header_bytes)
            .map_err(|e| C3dWriteError::WriteHeaderError(e))?;
        file.write_all(&parameter_bytes)
            .map_err(|e| C3dWriteError::WriteParametersError(e))?;
        file.write_all(&data_bytes)
            .map_err(|e| C3dWriteError::WriteDataError(e))?;
        file.sync_all()
            .map_err(|e| C3dWriteError::WriteError(file_name.clone(), e))?;
        Ok(self)
    }
}

fn read_header_bytes(file: &mut File) -> Result<[u8; 512], C3dParseError> {
    let mut header_bytes = [0u8; 512];
    file.read_exact(&mut header_bytes)
        .map_err(|e| C3dParseError::ReadError(e))?;
    Ok(header_bytes)
}

fn read_parameter_bytes(
    file: &mut File,
    header_bytes: &[u8; 512],
) -> Result<(Processor, Vec<u8>, usize), C3dParseError> {
    let parameter_start_block_index = header_bytes[0] as usize;

    let blocks_to_skip = parameter_start_block_index - 2;
    file.seek(SeekFrom::Current((512 * blocks_to_skip) as i64))
        .map_err(|e| C3dParseError::ReadError(e))?;

    let mut parameter_start_block = [0u8; 512];
    file.read_exact(&mut parameter_start_block)
        .map_err(|e| C3dParseError::ReadError(e))?;

    let processor = Processor::from_parameter_start_block(parameter_start_block)?;
    let data_start_block_index = processor.u16([header_bytes[16], header_bytes[17]]) as usize;

    let mut parameter_bytes_tail =
        Vec::with_capacity((data_start_block_index - parameter_start_block_index - 1) * 512)
            as Vec<u8>;

    for _ in 0..(data_start_block_index - parameter_start_block_index - 1) {
        let mut block = [0u8; 512];
        file.read_exact(&mut block)
            .map_err(|e| C3dParseError::ReadError(e))?;
        parameter_bytes_tail.extend(block.iter());
    }

    let parameter_bytes = [
        parameter_start_block.as_slice(),
        parameter_bytes_tail.as_slice(),
    ]
    .concat();

    Ok((processor, parameter_bytes, data_start_block_index))
}

fn read_data_bytes(mut file: File) -> Result<Vec<u8>, C3dParseError> {
    let mut data: Vec<u8> = Vec::new();

    file.read_to_end(&mut data)
        .map_err(|e| C3dParseError::ReadError(e))?;
    Ok(data)
}
