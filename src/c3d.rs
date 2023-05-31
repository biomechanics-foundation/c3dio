use crate::data::{
    parse_analog_data_float, parse_analog_data_int, parse_point_data_float, parse_point_data_int,
    DataFormat,
};
use crate::parameters::{parse_parameter_blocks, ParameterData};
use crate::parse::C3dParseError;
use crate::processor::{bytes_to_f32, bytes_to_u16, get_processor_type, ProcessorType};
use ndarray::{Array, Array2, Array3};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Events {
    pub supports_event_labels: bool,
    pub num_time_events: u8,
    pub event_times: Vec<f32>,
    pub event_display_flags: Vec<bool>,
    pub event_labels: Vec<String>,
}

#[derive(Debug, PartialEq)]
enum ParseState {
    Unparsed,
    BasicInfo,
    Header,
    Parameters,
    Data,
}

#[derive(Debug)]
pub struct C3d {
    pub file_path: PathBuf,
    pub file: File,
    parse_state: ParseState,
    pub header_read: bool,
    pub header_bytes: [u8; 512],
    pub parameter_read: bool,
    pub parameter_bytes: Vec<u8>,
    pub parameters: HashMap<String, HashMap<String, ParameterData>>,
    pub group_descriptions: HashMap<String, String>,
    pub parameter_descriptions: HashMap<String, String>,
    pub points: Array3<f32>,
    pub point_labels: Vec<String>,
    pub cameras: Array3<bool>,
    pub residuals: Array2<f32>,
    pub analog: Array2<f32>,
    pub analog_labels: Vec<String>,
    pub analog_channels: u16,
    pub analog_samples_per_frame: u16,
    pub data_format: DataFormat,
    pub parameter_start_block_index: usize,
    pub data_read: bool,
    pub data_bytes: Vec<u8>,
    pub data_start_block_index: usize,
    pub processor_type: ProcessorType,
    pub points_per_frame: u16,
    pub first_frame: u16,
    pub last_frame: u16,
    pub max_interpolation_gap: u16,
    pub scale_factor: f32,
    pub frame_rate: f32,
    pub events: Events,
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
            && self.data_read == other.data_read
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
    fn blank(file_name: &str) -> Result<C3d, C3dParseError> {
        let path = PathBuf::from(file_name);
        Ok(C3d {
            file_path: path,
            file: File::open(file_name).map_err(|e| C3dParseError::ReadError(e))?,
            parse_state: ParseState::Unparsed,
            header_bytes: [0; 512],
            parameter_bytes: Vec::new(),
            header_read: false,
            parameter_read: false,
            data_read: false,
            parameters: HashMap::new(),
            group_descriptions: HashMap::new(),
            parameter_descriptions: HashMap::new(),
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

    pub fn parse_basic_info(mut self) -> Result<C3d, C3dParseError> {
        self.header_bytes = [0u8; 512];
        self.file
            .read_exact(&mut self.header_bytes)
            .map_err(|e| C3dParseError::ReadError(e))?;
        self.header_read = true;

        self.parameter_start_block_index = self.header_bytes[0] as usize;

        let mut parameter_start_block = [0u8; 512];
        self.file
            .read_exact(&mut parameter_start_block)
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
            self.file
                .read_exact(&mut block)
                .map_err(|e| C3dParseError::ReadError(e))?;
            parameter_bytes_tail.extend(block.iter());
        }
        self.parameter_read = true;

        self.parameter_bytes = [
            parameter_start_block.as_slice(),
            parameter_bytes_tail.as_slice(),
        ]
        .concat();

        Ok(self)
    }

    pub fn parse_header(mut self) -> Result<C3d, C3dParseError> {
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

    pub fn parse_parameters(mut self) -> Result<C3d, C3dParseError> {
        let (parameters, group_descriptions, parameter_descriptions) =
            parse_parameter_blocks(&self.parameter_bytes, &self.processor_type)?;

        self.parameters = parameters;
        self.group_descriptions = group_descriptions;
        self.parameter_descriptions = parameter_descriptions;

        let point_scale_data = self.get_parameter_value("POINT", "SCALE");
        if let Some(ParameterData::Float(point_scale)) = point_scale_data {
            let point_scale = point_scale.first();
            if let Some(&point_scale) = point_scale {
                if point_scale < 0.0 {
                    self.data_format = DataFormat::Float;
                } else {
                    self.data_format = DataFormat::Integer;
                }
            }
        } else {
            //self.data_format = DataFormat::Integer;
            return Err(C3dParseError::MissingPointScale);
        }

        Ok(self)
    }

    pub fn get_parameter_value(
        &self,
        group_name: &str,
        parameter_name: &str,
    ) -> Option<&ParameterData> {
        let group = self.parameters.get(group_name)?;
        Some(group.get(parameter_name)?)
    }

    pub fn parse_data(mut self) -> Result<C3d, C3dParseError> {
        let bytes_per_point = match self.data_format {
            DataFormat::Float => 16,
            DataFormat::Integer => 8,
            DataFormat::Unknown => {
                return Err(C3dParseError::UnknownDataFormat);
            }
        };

        let point_bytes_per_frame = bytes_per_point * self.points_per_frame;

        let bytes_per_analog_point = match self.data_format {
            DataFormat::Float => 4,
            DataFormat::Integer => 2,
            DataFormat::Unknown => {
                return Err(C3dParseError::UnknownDataFormat);
            }
        };

        let analog_bytes_per_frame =
            bytes_per_analog_point * self.analog_samples_per_frame;

        let bytes_per_frame = point_bytes_per_frame + analog_bytes_per_frame;

        let num_frames = (self.last_frame as usize - self.first_frame as usize) + 1;

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

        self.data_bytes = Vec::new();
        self.file
            .read_to_end(&mut self.data_bytes)
            .map_err(|e| C3dParseError::ReadError(e))?;

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
            for j in 0..self.analog_channels{
                let start = j * bytes_per_analog_point * self.analog_samples_per_frame / self.analog_channels;
                let end = start + bytes_per_analog_point * self.analog_samples_per_frame / self.analog_channels;
                let analog_slice = &analog_frame_data[start as usize..end as usize];
                let analog_data = match self.data_format {
                    DataFormat::Float => parse_analog_data_float(
                        analog_slice,
                        (self.analog_samples_per_frame / self.analog_channels) as usize,
                        &self.processor_type,
                    ),
                    DataFormat::Integer => parse_analog_data_int(
                        analog_slice,
                        (self.analog_samples_per_frame / self.analog_channels) as usize,
                        &self.processor_type,
                    ),
                    DataFormat::Unknown => {
                        return Err(C3dParseError::UnknownDataFormat);
                    }
                };
                for k in 0..self.analog_samples_per_frame / self.analog_channels {
                    *analog_iter.next().unwrap() = analog_data[k as usize];
                }
            }
        }

        Ok(self)
    }

    pub fn from_file(file_name: &str) -> Result<C3d, C3dParseError> {
        let c3d = C3d::blank(file_name)?
            .parse_basic_info()?
            .parse_header()?
            .parse_parameters()?
            .parse_data()?;

        Ok(c3d)
    }

    pub fn header(file_name: &str) -> Result<C3d, C3dParseError> {
        let c3d = C3d::blank(file_name)?.parse_basic_info()?.parse_header()?;

        Ok(c3d)
    }

    pub fn parameters(file_name: &str) -> Result<C3d, C3dParseError> {
        let c3d = C3d::blank(file_name)?
            .parse_basic_info()?
            .parse_header()?
            .parse_parameters()?;

        Ok(c3d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_advanced_realtime_tracking() {
        // Advanced Realtime Tracking GmbH
        assert!(
            C3d::from_file("res/Sample00/Advanced Realtime Tracking GmbH/arthuman-sample.c3d").is_ok()
        );
        assert!(C3d::from_file(
            "res/Sample00/Advanced Realtime Tracking GmbH/arthuman-sample-fingers.c3d"
        )
        .is_ok());
    }

    #[test]
    fn test_parse_codamotion() {
        // Codamotion
        assert!(C3d::from_file("res/Sample00/Codamotion/codamotion_gaitwands_19970212.c3d").is_ok());
        assert!(C3d::from_file("res/Sample00/Codamotion/codamotion_gaitwands_20150204.c3d").is_ok());
    }

    #[test]
    fn test_parse_cometa() {
        // Cometa
        assert!(C3d::from_file("res/Sample00/Cometa Systems/EMG Data Cometa.c3d").is_ok());
    }

    #[test]
    fn test_parse_innovative_sports_training() {
        // Innovative Sports Training
        assert!(C3d::from_file("res/Sample00/Innovative Sports Training/Gait with EMG.c3d").is_ok());
        assert!(C3d::from_file("res/Sample00/Innovative Sports Training/Static Pose.c3d").is_ok());
    }

    #[test]
    fn test_parse_motion_analysis_corporation() {
        // Motion Analysis Corporation
        assert!(C3d::from_file("res/Sample00/Motion Analysis Corporation/Sample_Jump2.c3d").is_ok());
        assert!(C3d::from_file("res/Sample00/Motion Analysis Corporation/Walk1.c3d").is_ok());
    }

    #[test]
    fn test_parse_nexgen_ergonomics() {
        // NexGen Ergonomics
        assert!(C3d::from_file("res/Sample00/NexGen Ergonomics/test1.c3d").is_ok());
    }

    #[test]
    fn test_parse_vicon_motion_systems() {
        // Vicon Motion Systems
        assert!(C3d::from_file("res/Sample00/Vicon Motion Systems/TableTennis.c3d").is_ok());
        assert!(C3d::from_file(
            "res/Sample00/Vicon Motion Systems/pyCGM2 lower limb CGM24 Walking01.c3d"
        )
        .is_ok());
    }
}
