use crate::data::{
    parse_analog_data_float, parse_analog_data_int, parse_point_data_float, parse_point_data_int,
    DataFormat,
};
use crate::parameters::{parse_parameter_blocks, ParameterData};

use crate::{C3d, C3dParseError};

use crate::processor::{bytes_to_f32, bytes_to_u16, get_processor_type, ProcessorType};
use ndarray::{Array, Array2, Array3};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Events {
    pub supports_event_labels: bool,
    pub num_time_events: u8,
    pub event_times: Vec<f32>,
    pub event_display_flags: Vec<bool>,
    pub event_labels: Vec<String>,
}

impl PartialEq for C3d {
    fn eq(&self, other: &Self) -> bool {
        self.points == other.points
            && self.cameras == other.cameras
            && self.residuals == other.residuals
            && self.analog == other.analog
            && self.point_labels == other.point_labels
            && self.analog_labels == other.analog_labels
            && self.parameter_start_block_index == other.parameter_start_block_index
            && self.data_start_block_index == other.data_start_block_index
            && self.points_per_frame == other.points_per_frame
            && self.analog_samples_per_frame == other.analog_samples_per_frame
            && self.first_frame == other.first_frame
            && self.last_frame == other.last_frame
            && self.max_interpolation_gap == other.max_interpolation_gap
            && self.scale_factor.abs() == other.scale_factor.abs()
            && self.analog_channels == other.analog_channels
            && self.frame_rate == other.frame_rate
    }
}

impl C3d {
    pub fn load(file_name: &str) -> Result<C3d, C3dParseError> {
        let c3d = C3d::empty()?
            .open_file(file_name)?
            .parse_basic_info()?
            .parse_header()?
            .parse_parameters()?
            .parse_data()?;

        Ok(c3d)
    }

    pub fn load_header(file_name: &str) -> Result<C3d, C3dParseError> {
        let c3d = C3d::empty()?
            .open_file(file_name)?
            .parse_basic_info()?
            .parse_header()?;

        Ok(c3d)
    }

    pub fn load_parameters(file_name: &str) -> Result<C3d, C3dParseError> {
        let c3d = C3d::empty()?
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
                    for row in data.rows() {
                        let mut string = String::new();
                        for c in row {
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

    pub fn empty() -> Result<C3d, C3dParseError> {
        let path = PathBuf::from("");
        Ok(C3d {
            file_path: path,
            file: None,
            header_bytes: [0; 512],
            parameter_bytes: Vec::new(),
            parameters: HashMap::new(),
            group_descriptions: HashMap::new(),
            points: Array3::zeros((0, 0, 0)),
            cameras: Array3::from_elem((0, 0, 0), false),
            residuals: Array2::zeros((0, 0)),
            analog: Array2::zeros((0, 0)),
            point_labels: Vec::new(),
            analog_labels: Vec::new(),
            analog_channels: 0,
            data_format: DataFormat::Unknown,
            parameter_start_block_index: 0,
            data_start_block_index: 0,
            data_bytes: Vec::new(),
            processor_type: ProcessorType::Unknown,
            points_per_frame: 0,
            analog_samples_per_frame: 0,
            first_frame: 0,
            last_frame: 0,
            max_interpolation_gap: 0,
            scale_factor: 0.0,
            frame_rate: 0.0,
            events: Events {
                supports_event_labels: false,
                num_time_events: 0,
                event_times: Vec::new(),
                event_display_flags: Vec::new(),
                event_labels: Vec::new(),
            },
        })
    }

    pub(super) fn open_file(mut self, file_name: &str) -> Result<C3d, C3dParseError> {
        self.file_path = PathBuf::from(file_name);
        self.file = Some(File::open(&self.file_path).map_err(|e| C3dParseError::ReadError(e))?);
        Ok(self)
    }

    pub(super) fn parse_basic_info(mut self) -> Result<C3d, C3dParseError> {
        self.header_bytes = [0u8; 512];

        if self.file.is_none() {
            return Err(C3dParseError::FileNotOpen);
        }

        let file = self.file.as_mut().unwrap();
        file.read_exact(&mut self.header_bytes)
            .map_err(|e| C3dParseError::ReadError(e))?;

        self.parameter_start_block_index = self.header_bytes[0] as usize;

        let blocks_to_skip = self.parameter_start_block_index - 2;

        file.seek(SeekFrom::Current((512 * blocks_to_skip) as i64))
            .map_err(|e| C3dParseError::ReadError(e))?;

        let mut parameter_start_block = [0u8; 512];
        file.read_exact(&mut parameter_start_block)
            .map_err(|e| C3dParseError::ReadError(e))?;

        self.processor_type = match get_processor_type(parameter_start_block) {
            Ok(processor_type) => processor_type,
            Err(e) => return Err(e),
        };

        self.data_start_block_index = bytes_to_u16(
            [self.header_bytes[16], self.header_bytes[17]],
            &self.processor_type,
        ) as usize;

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

    pub(super) fn parse_header(mut self) -> Result<C3d, C3dParseError> {
        let processor_type = self.processor_type.clone();
        self.points_per_frame =
            bytes_to_u16(self.header_bytes[2..4].try_into().unwrap(), &processor_type);
        self.analog_samples_per_frame =
            bytes_to_u16(self.header_bytes[4..6].try_into().unwrap(), &processor_type);
        self.first_frame =
            bytes_to_u16(self.header_bytes[6..8].try_into().unwrap(), &processor_type);
        self.last_frame = bytes_to_u16(
            self.header_bytes[8..10].try_into().unwrap(),
            &processor_type,
        );
        self.max_interpolation_gap = bytes_to_u16(
            self.header_bytes[10..12].try_into().unwrap(),
            &processor_type,
        );
        self.scale_factor = bytes_to_f32(
            self.header_bytes[12..16].try_into().unwrap(),
            &processor_type,
        );
        self.analog_channels = bytes_to_u16(
            self.header_bytes[18..20].try_into().unwrap(),
            &processor_type,
        );
        self.frame_rate = bytes_to_f32(
            self.header_bytes[20..24].try_into().unwrap(),
            &processor_type,
        );
        let supports_event_labels_value = bytes_to_u16(
            self.header_bytes[300..302].try_into().unwrap(),
            &processor_type,
        );
        let supports_event_labels = supports_event_labels_value == 0x3039;
        let num_time_events: [u8; 1] = self.header_bytes[302..303].try_into().unwrap();
        let num_time_events = num_time_events[0];
        let mut event_times = Vec::new();

        // event times start at byte 306
        for i in 0..18 {
            let start = 304 + (i * 4);
            let end = start + 4;
            event_times.push(bytes_to_f32(
                self.header_bytes[start..end].try_into().unwrap(),
                &processor_type,
            ));
        }

        // event display flags start at byte 378
        let mut event_display_flags = Vec::new();

        for i in 0..18 {
            let index = 378 + i;
            let flag: [u8; 1] = self.header_bytes[index..index + 1].try_into().unwrap();
            if flag[0] == 0x01 {
                event_display_flags.push(true);
            } else {
                event_display_flags.push(false);
            }
        }

        // event labels start at byte 398
        let mut event_labels = Vec::new();

        for i in 0..18 {
            let start = 398 + (i * 4);
            let end = start + 4;

            let label_bytes: [u8; 4] = self.header_bytes[start..end].try_into().unwrap();

            let label_chars = label_bytes
                .iter()
                .map(|b| *b as char)
                .collect::<Vec<char>>();

            event_labels.push(label_chars.into_iter().collect::<String>());
        }

        self.events = Events {
            supports_event_labels,
            num_time_events,
            event_times,
            event_display_flags,
            event_labels,
        };

        Ok(self)
    }

    pub(super) fn parse_parameters(mut self) -> Result<C3d, C3dParseError> {
        let (parameters, group_descriptions) =
            parse_parameter_blocks(&self.parameter_bytes, &self.processor_type)?;

        self.parameters = parameters;
        self.group_descriptions = group_descriptions;

        let point_scale = self.get_parameter_float("POINT", "SCALE");
        if let Some(point_scale) = point_scale {
            if point_scale < 0.0 {
                self.data_format = DataFormat::Float;
            } else {
                self.data_format = DataFormat::Integer;
            }
        } else {
            self.data_format = DataFormat::Integer;
            //return Err(C3dParseError::MissingPointScale);
        }

        Ok(self)
    }

    pub(super) fn parse_data(mut self) -> Result<C3d, C3dParseError> {
        let bytes_per_point = match self.data_format {
            DataFormat::Float => 16,
            DataFormat::Integer => 8,
            DataFormat::Unknown => 0,
        };

        let point_bytes_per_frame = (bytes_per_point * self.points_per_frame) as usize;

        self.data_bytes = Vec::new();

        if self.file.is_none() {
            return Err(C3dParseError::FileNotOpen);
        }

        let file = self.file.as_mut().unwrap();
        file.read_to_end(&mut self.data_bytes)
            .map_err(|e| C3dParseError::ReadError(e))?;

        let num_frames = (self.last_frame as usize - self.first_frame as usize) + 1;
        let mut num_frames = match self.get_parameter_data("POINT", "FRAMES") {
            Some(ParameterData::Integer(frames)) => {
                let frames = frames.first();
                if let Some(&frames) = frames {
                    frames as u16 as usize
                } else {
                    num_frames
                }
            }
            Some(ParameterData::Float(frames)) => {
                let frames = frames.first();
                if let Some(&frames) = frames {
                    frames as u16 as usize
                } else {
                    num_frames
                }
            }
            _ => num_frames,
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

                let start_field = *start_one as u16 as usize + (*start_two as u16 * 65535) as usize;
                let end_field = *end_one as u16 as usize + (*end_two as u16 * 65535) as usize;
                num_frames = end_field - start_field + 1;
            }
        }

        let bytes_per_analog_point = match self.data_format {
            DataFormat::Float => 4,
            DataFormat::Integer => 2,
            DataFormat::Unknown => {
                let estimated_analog_bytes = match self.analog_samples_per_frame {
                    0 => 0,
                    _ => {
                        self.data_bytes.len()
                            / (self.analog_samples_per_frame as usize * num_frames)
                    }
                };
                match estimated_analog_bytes {
                    2 => 2,
                    4 => 4,
                    _ => return Err(C3dParseError::UnknownDataFormat),
                }
                //return Err(C3dParseError::UnknownDataFormat);
            }
        } as usize;

        let analog_bytes_per_frame =
            bytes_per_analog_point * self.analog_samples_per_frame as usize;

        let bytes_per_frame = point_bytes_per_frame + analog_bytes_per_frame;

        let num_frames = match self.data_bytes.len() < num_frames * bytes_per_frame as usize {
            true => {
                let num_frames = self.data_bytes.len() / bytes_per_frame as usize;
                num_frames
            }
            false => num_frames,
            //return Err(C3dParseError::NotEnoughData);
        };

        let mut point_data: Array3<f32> =
            Array::zeros((num_frames as usize, self.points_per_frame as usize, 3));
        let mut analog_data: Array2<f32> =
            Array::zeros((num_frames as usize, self.analog_samples_per_frame as usize));
        let mut cameras: Array3<bool> = Array::from_elem(
            (num_frames as usize, self.points_per_frame as usize, 7),
            false,
        );
        let mut residual: Array2<f32> =
            Array::zeros((num_frames as usize, self.points_per_frame as usize));

        let mut point_iter = point_data.iter_mut();
        let mut analog_iter = analog_data.iter_mut();
        let mut camera_iter = cameras.iter_mut();
        let mut residual_iter = residual.iter_mut();

        let analog_samples_per_channel_per_frame = if self.analog_channels > 0 {
            (self.analog_samples_per_frame / self.analog_channels) as usize
        } else {
            0
        };

        for i in 0..num_frames {
            let start = i * bytes_per_frame as usize;
            let end = start + bytes_per_frame as usize;
            let frame = &self.data_bytes[start as usize..end as usize];
            let point_frame_data = &frame[0..point_bytes_per_frame as usize];
            let analog_frame_data = &frame[point_bytes_per_frame as usize..];
            for j in 0..self.points_per_frame {
                let start = j * bytes_per_point;
                let end = start + bytes_per_point;
                let point_slice = &point_frame_data[start as usize..end as usize];
                let (x, y, z, cameras, residual) = match self.data_format {
                    DataFormat::Float => parse_point_data_float(point_slice, &self.processor_type),
                    DataFormat::Integer => {
                        parse_point_data_int(point_frame_data, &self.processor_type)
                    }
                    DataFormat::Unknown => {
                        return Err(C3dParseError::UnknownDataFormat);
                    }
                };
                *point_iter.next().unwrap() = x;
                *point_iter.next().unwrap() = y;
                *point_iter.next().unwrap() = z;
                for k in 0..7 {
                    *camera_iter.next().unwrap() = cameras[k];
                }
                *residual_iter.next().unwrap() = residual;
            }
            for j in 0..self.analog_channels as usize {
                let start = j * bytes_per_analog_point * analog_samples_per_channel_per_frame;
                let end = start + bytes_per_analog_point * analog_samples_per_channel_per_frame;
                let analog_slice = &analog_frame_data[start as usize..end as usize];
                let analog_data = match self.data_format {
                    DataFormat::Float => parse_analog_data_float(
                        analog_slice,
                        analog_samples_per_channel_per_frame,
                        &self.processor_type,
                    ),
                    DataFormat::Integer => parse_analog_data_int(
                        analog_slice,
                        analog_samples_per_channel_per_frame,
                        &self.processor_type,
                    ),
                    DataFormat::Unknown => {
                        return Err(C3dParseError::UnknownDataFormat);
                    }
                };
                for k in 0..analog_samples_per_channel_per_frame {
                    *analog_iter.next().unwrap() = analog_data[k as usize];
                }
            }
        }

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
    let c3d = match C3d::empty() {
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
