//! Implements the Points struct and methods for parsing and writing point data.
use crate::data::{get_analog_bytes_per_frame, get_point_bytes_per_frame, DataFormat, MarkerPoint};
use crate::parameters::{Parameter, ParameterData, Parameters};
use crate::processor::Processor;
use crate::{C3dParseError, C3dWriteError};
use grid::Grid;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

#[derive(Clone)]
pub struct Points {
    parsed_header: bool,
    pub points: Grid<MarkerPoint>,
    pub labels: Vec<String>,
    pub descriptions: Vec<String>,
    pub units: [char; 4],
    pub x_screen: Option<[char; 2]>,
    pub y_screen: Option<[char; 2]>,
    pub frame_rate: f32,
    pub scale_factor: f32,
    pub max_interpolation_gap: u16,
    pub first_frame: u16,
    pub last_frame: u16,
    pub format: DataFormat,
}

impl PartialEq for Points {
    fn eq(&self, other: &Self) -> bool {
        self.parsed_header == other.parsed_header
            && self.points.flatten() == other.points.flatten()
            && self.labels == other.labels
            && self.descriptions == other.descriptions
            && self.units == other.units
            && self.x_screen == other.x_screen
            && self.y_screen == other.y_screen
            && self.frame_rate == other.frame_rate
            && self.scale_factor == other.scale_factor
            && self.max_interpolation_gap == other.max_interpolation_gap
            && self.first_frame == other.first_frame
            && self.last_frame == other.last_frame
        //            && self.format == other.format
    }
}

impl Debug for Points {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Points")
            // .field("points", &self.points)
            .field("labels", &self.labels)
            .field("descriptions", &self.descriptions)
            .field("units", &self.units)
            .field("x_screen", &self.x_screen)
            .field("y_screen", &self.y_screen)
            .field("frame_rate", &self.frame_rate)
            .field("scale_factor", &self.scale_factor)
            .field("max_interpolation_gap", &self.max_interpolation_gap)
            .field("first_frame", &self.first_frame)
            .field("last_frame", &self.last_frame)
            .field("format", &self.format)
            .finish()
    }
}

impl Deref for Points {
    type Target = Grid<MarkerPoint>;

    fn deref(&self) -> &Self::Target {
        &self.points
    }
}

impl DerefMut for Points {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.points
    }
}

impl ToString for Points {
    fn to_string(&self) -> String {
        let mut string = String::new();
        string.push_str("Marker Data:\n");
        string.push_str(&format!("Data Size: {:?}\n", self.points.size()));
        string.push_str(&format!("Labels: {:?}\n", self.labels));
        string.push_str(&format!("Descriptions: {:?}\n", self.descriptions));
        string.push_str(&format!("Units: {:?}\n", self.units));
        string.push_str(&format!("X Screen: {:?}\n", self.x_screen));
        string.push_str(&format!("Y Screen: {:?}\n", self.y_screen));
        string.push_str(&format!("Frame Rate: {}\n", self.frame_rate));
        string.push_str(&format!("Scale Factor: {}\n", self.scale_factor));
        string.push_str(&format!(
            "Max Interpolation Gap: {}\n",
            self.max_interpolation_gap
        ));
        string.push_str(&format!("First Frame: {}\n", self.first_frame));
        string.push_str(&format!("Last Frame: {}\n", self.last_frame));
        string.push_str(&format!("Format: {:?}\n", self.format));
        string
    }
}

impl Default for Points {
    fn default() -> Self {
        Points {
            parsed_header: false,
            points: Grid::new(0, 0),
            labels: Vec::new(),
            descriptions: Vec::new(),
            units: [' '; 4],
            x_screen: None,
            y_screen: None,
            frame_rate: 0.0,
            scale_factor: 0.0,
            max_interpolation_gap: 0,
            first_frame: 0,
            last_frame: 0,
            format: DataFormat::Float,
        }
    }
}

impl Points {
    pub(crate) fn new() -> Self {
        Points::default()
    }

    pub(crate) fn parse_header(header: &[u8; 512], processor: &Processor) -> Self {
        let mut points = Points::new();
        let num_markers = processor.u16([header[2], header[3]]);
        points.first_frame = processor.u16([header[6], header[7]]);
        points.last_frame = processor.u16([header[8], header[9]]);
        points.points = Grid::new(
            (points.last_frame - points.first_frame + 1) as usize,
            num_markers as usize,
        );
        points.max_interpolation_gap = processor.u16([header[10], header[11]]);
        let scale_factor = processor.f32([header[12], header[13], header[14], header[15]]);
        if scale_factor <= 0.0 {
            points.format = DataFormat::Float;
        } else {
            points.format = DataFormat::Integer;
        }
        points.scale_factor = scale_factor.abs();
        points.frame_rate = processor.f32([header[20], header[21], header[22], header[23]]);
        points.parsed_header = true;
        points
    }

    pub(crate) fn parse(
        &mut self,
        data_bytes: &[u8],
        parameters: &mut Parameters,
        processor: &Processor,
        analog_samples_per_frame: u16,
    ) -> Result<(&mut Self, usize), C3dParseError> {
        if !self.parsed_header {
            return Err(C3dParseError::HeaderNotParsed);
        }
        let (point_frames, actual_start_field, actual_end_field, long_frames) =
            self.get_point_parameters(parameters)?;
        let num_frames = self.calc_num_frames(
            data_bytes,
            point_frames,
            actual_start_field,
            actual_end_field,
            long_frames,
            analog_samples_per_frame,
        )?;
        self.parse_points(data_bytes, processor, analog_samples_per_frame, num_frames)?;
        Ok((self, num_frames))
    }

    pub(crate) fn write_parameters(
        &self,
        processor: &Processor,
        group_names_to_ids: &HashMap<String, usize>,
        num_frames: usize,
    ) -> Result<Vec<u8>, C3dWriteError> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend(Parameter::integer(self.cols() as i16).write(
            processor,
            "USED".to_string(),
            group_names_to_ids["POINT"],
            false,
        )?);
        bytes.extend(Parameter::chars(self.units.to_vec())?.write(
            processor,
            "UNITS".to_string(),
            group_names_to_ids["POINT"],
            false,
        )?);
        bytes.extend(Parameter::strings(self.labels.clone()).write(
            processor,
            "LABELS".to_string(),
            group_names_to_ids["POINT"],
            false,
        )?);
        bytes.extend(Parameter::strings(self.descriptions.clone()).write(
            processor,
            "DESCRIPTIONS".to_string(),
            group_names_to_ids["POINT"],
            false,
        )?);
        if self.x_screen.is_some() {
            bytes.extend(Parameter::chars(self.x_screen.unwrap().to_vec())?.write(
                processor,
                "X_SCREEN".to_string(),
                group_names_to_ids["POINT"],
                false,
            )?);
        }
        if self.y_screen.is_some() {
            bytes.extend(Parameter::chars(self.y_screen.unwrap().to_vec())?.write(
                processor,
                "Y_SCREEN".to_string(),
                group_names_to_ids["POINT"],
                false,
            )?);
        }
        bytes.extend(Parameter::float(num_frames as f32).write(
            processor,
            "FRAMES".to_string(),
            group_names_to_ids["POINT"],
            false,
        )?);
        Ok(bytes)
    }

    pub(crate) fn write_frame(&self, frame: usize, processor: &Processor) -> Vec<u8> {
        let mut bytes = Vec::new();
        if frame >= self.points.rows() {
            return bytes;
        }
        for (column, _) in self.iter_cols().enumerate() {
            match self.format {
                DataFormat::Float => {
                    let x = processor.f32_to_bytes(self.points[frame][column][0] as f32);
                    let y = processor.f32_to_bytes(self.points[frame][column][1] as f32);
                    let z = processor.f32_to_bytes(self.points[frame][column][2] as f32);
                    bytes.extend_from_slice(&x);
                    bytes.extend_from_slice(&y);
                    bytes.extend_from_slice(&z);
                    let residual =
                        (self.points[frame][column].residual / self.scale_factor).round() as i8;
                    let cameras = self.points[frame][column].cameras_as_byte();
                    let cameras_and_residual =
                        processor
                            .f32_to_bytes(i16::from_be_bytes([cameras, residual as u8]) as f32);
                    bytes.extend_from_slice(&cameras_and_residual);
                }
                DataFormat::Integer => {
                    let x = processor.i16_to_bytes(
                        (self.points[frame][column][0] / self.scale_factor).round() as i16,
                    );
                    let y = processor.i16_to_bytes(
                        (self.points[frame][column][1] / self.scale_factor).round() as i16,
                    );
                    let z = processor.i16_to_bytes(
                        (self.points[frame][column][2] / self.scale_factor).round() as i16,
                    );
                    bytes.extend_from_slice(&x);
                    bytes.extend_from_slice(&y);
                    bytes.extend_from_slice(&z);
                    let residual =
                        (self.points[frame][column].residual / self.scale_factor).round() as i16;
                    let cameras = self.points[frame][column].cameras_as_byte();
                    if residual > 0 {
                        bytes.extend(
                            processor.u16_to_bytes(u16::from_be_bytes([cameras, residual as u8])),
                        );
                    } else {
                        let cameras = cameras | 0x80;
                        bytes.extend(
                            processor.u16_to_bytes(u16::from_be_bytes([cameras, residual as u8])),
                        );
                    }
                }
            }
        }
        bytes
    }

    fn get_point_parameters(
        &mut self,
        parameters: &mut Parameters,
    ) -> Result<(usize, Option<usize>, Option<usize>, Option<usize>), C3dParseError> {
        let point_frames = match parameters.remove("POINT", "FRAMES") {
            Some(parameter) => match &parameter.data {
                ParameterData::Integer(frames) => frames[0] as u16 as usize,
                ParameterData::Float(frames) => frames[0] as usize,
                _ => 0,
            },
            _ => 0,
        };
        let (actual_start_field, actual_end_field) = get_actual_start_and_end_fields(parameters)?;
        let long_frames = parameters.remove("POINT", "LONG_FRAMES");
        let long_frames = match long_frames {
            Some(frames) => match &frames.data {
                ParameterData::Integer(frames) => Some(frames[0] as u16 as usize),
                ParameterData::Float(frames) => Some(frames[0] as usize),
                _ => Some(0),
            },
            None => None,
        };
        let used = parameters.remove("POINT", "USED");
        let mut is_none_or_zero = used.is_none();
        if !is_none_or_zero {
            let used = used.unwrap();
            match &used.data {
                ParameterData::Integer(used) => {
                    is_none_or_zero = used[0] == 0;
                }
                ParameterData::Float(used) => {
                    is_none_or_zero = used[0] == 0.0;
                }
                _ => {}
            }
        }
        if is_none_or_zero {
            Ok((
                point_frames,
                actual_start_field,
                actual_end_field,
                long_frames,
            ))
        } else {
            self.labels = parameters
                .remove_or_err("POINT", "LABELS")?
                .as_ref()
                .try_into()?;
            self.descriptions = parameters
                .remove("POINT", "DESCRIPTIONS")
                .unwrap_or(Parameter::strings(vec![" ".to_string()]))
                .as_ref()
                .try_into()?;
            self.units = parameters
                .remove_or_err("POINT", "UNITS")?
                .as_ref()
                .try_into()?;
            let x_screen = parameters.remove("POINT", "X_SCREEN");
            self.x_screen = match x_screen {
                Some(parameter) => Some(parameter.as_ref().try_into()?),
                None => None,
            };
            let y_screen = parameters.remove("POINT", "Y_SCREEN");
            self.y_screen = match y_screen {
                Some(parameter) => Some(parameter.as_ref().try_into()?),
                None => None,
            };
            // some c3d files don't have a POINT:RATE parameter
            // but it is required by the c3d spec
            let rate = parameters.remove("POINT", "RATE");
            match rate {
                Some(rate) => {
                    let rate: f32 = rate.as_ref().try_into()?;
                    if rate != self.frame_rate {
                        return Err(C3dParseError::FrameRateMismatch(self.frame_rate, rate));
                    }
                }
                None => {}
            }
            let scale_factor = parameters.remove("POINT", "SCALE");
            match scale_factor {
                Some(scale_factor) => {
                    let scale_factor: f32 = scale_factor.as_ref().try_into()?;
                    if scale_factor.abs() != self.scale_factor {
                        return Err(C3dParseError::ScaleFactorMismatch(
                            self.scale_factor,
                            scale_factor,
                        ));
                    }
                }
                None => {}
            }
            Ok((
                point_frames,
                actual_start_field,
                actual_end_field,
                long_frames,
            ))
        }
    }

    fn calc_num_frames(
        &mut self,
        data_bytes: &[u8],
        point_frames: usize,
        actual_start_field: Option<usize>,
        actual_end_field: Option<usize>,
        long_frames: Option<usize>,
        analog_samples_per_frame: u16,
    ) -> Result<usize, C3dParseError> {
        let mut num_frames = (self.last_frame as usize - self.first_frame as usize) + 1;
        if num_frames != point_frames {
            num_frames = point_frames;
            //    return Err(C3dParseError::NumFramesMismatch(
            //        self.point_frames,
            //        num_frames,
            //    ));
        }
        if actual_start_field.is_some() && actual_end_field.is_some() {
            if (actual_end_field.unwrap() - actual_start_field.unwrap() + 1) as usize != num_frames
            {
                num_frames = (actual_end_field.unwrap() - actual_start_field.unwrap() + 1) as usize;
            }
        }
        if long_frames.is_some() {
            if long_frames.unwrap() > num_frames {
                num_frames = long_frames.unwrap();
            }
        }
        let point_bytes_per_frame = get_point_bytes_per_frame(&self.format, self.cols()) as usize;

        let analog_bytes_per_frame =
            get_analog_bytes_per_frame(&self.format, analog_samples_per_frame)?;

        let bytes_per_frame = point_bytes_per_frame + analog_bytes_per_frame;
        num_frames = match data_bytes.len() < num_frames * bytes_per_frame as usize {
            true => {
                let num_frames = data_bytes.len() / bytes_per_frame as usize;
                num_frames
            }
            false => num_frames,
            //return Err(C3dParseError::NotEnoughData);
        };
        Ok(num_frames)
    }

    fn parse_points(
        &mut self,
        data_bytes: &[u8],
        processor: &Processor,
        analog_samples_per_frame: u16,
        num_frames: usize,
    ) -> Result<&mut Self, C3dParseError> {
        let mut point_data = Grid::new(num_frames, self.cols() as usize);

        let point_bytes_per_frame = get_point_bytes_per_frame(&self.format, self.cols()) as usize;
        let analog_bytes_per_frame =
            get_analog_bytes_per_frame(&self.format, analog_samples_per_frame)?;
        let bytes_per_frame = point_bytes_per_frame + analog_bytes_per_frame;
        let bytes_per_point: u16 = match self.cols() {
            0 => 0,
            _ => point_bytes_per_frame as u16 / self.cols() as u16,
        };

        for i in 0..point_data.rows() {
            let start = i * bytes_per_frame as usize;
            let end = start + bytes_per_frame as usize;
            let point_frame_data = &data_bytes[start..end - analog_bytes_per_frame as usize];
            for j in 0..self.cols() as u16 {
                let start = j * bytes_per_point;
                let end = start + bytes_per_point;
                let point_slice = &point_frame_data[start as usize..end as usize];
                let mut point = match self.format {
                    DataFormat::Float => parse_point_data_float(point_slice, processor),
                    DataFormat::Integer => parse_point_data_int(point_slice, processor),
                };
                let point = match self.format {
                    DataFormat::Integer => point
                        .scale(self.scale_factor)
                        .scale_residual(self.scale_factor),
                    DataFormat::Float => point.scale_residual(self.scale_factor),
                };
                point_data[i][j as usize] = point;
            }
        }
        self.points = point_data;

        Ok(self)
    }
}

fn get_actual_start_and_end_fields(
    parameters: &mut Parameters,
) -> Result<(Option<usize>, Option<usize>), C3dParseError> {
    let end_field = parameters.remove("TRIAL", "ACTUAL_END_FIELD");
    let actual_end_field = if end_field.is_some() {
        let end_field: Vec<i16> = end_field.unwrap().as_ref().try_into()?;
        if end_field.len() != 2 {
            None
        } else {
            Some(end_field[0] as u16 as usize + (end_field[1] as u16 * 65535) as usize)
        }
    } else {
        None
    };
    let start_field = parameters.remove("TRIAL", "ACTUAL_START_FIELD");
    let actual_start_field = if start_field.is_some() {
        let start_field: Vec<i16> = start_field.unwrap().as_ref().try_into()?;
        if start_field.len() != 2 {
            None
        } else {
            Some(start_field[0] as u16 as usize + (start_field[1] as u16 * 65535) as usize)
        }
    } else {
        None
    };
    Ok((actual_start_field, actual_end_field))
}

fn parse_point_data_float(point_frame_data: &[u8], processor: &Processor) -> MarkerPoint {
    let x = processor.f32(point_frame_data[0..4].try_into().unwrap());
    let y = processor.f32(point_frame_data[4..8].try_into().unwrap());
    let z = processor.f32(point_frame_data[8..12].try_into().unwrap());
    let cameras_and_residual = processor.f32(point_frame_data[12..16].try_into().unwrap()) as i16;
    let cameras_and_residual = i16::from_be_bytes(cameras_and_residual.to_be_bytes());
    //let cameras_and_residual = processor.i16([
    //    (cameras_and_residual >> 8) as u8,
    //    cameras_and_residual as u8,
    //]);
    let cameras = byte_to_bool((cameras_and_residual >> 8) as u8);
    let residual = (cameras_and_residual & 0xFF) as i8 as f32;
    MarkerPoint {
        point: [x, y, z],
        cameras,
        residual,
    }
}

fn parse_point_data_int(point_frame_data: &[u8], processor: &Processor) -> MarkerPoint {
    let x = processor.i16(point_frame_data[0..2].try_into().unwrap());
    let y = processor.i16(point_frame_data[2..4].try_into().unwrap());
    let z = processor.i16(point_frame_data[4..6].try_into().unwrap());
    let cameras_and_residual = processor.i16(point_frame_data[6..8].try_into().unwrap());
    let cameras = byte_to_bool((cameras_and_residual >> 8) as u8);
    let residual = cameras_and_residual as u8;
    // if the first bit in the i16 is 0, then the residual is positive
    if cameras_and_residual >> 8 as i8 >= 0 {
        MarkerPoint {
            point: [x as f32, y as f32, z as f32],
            cameras,
            residual: residual as f32,
        }
    } else {
        MarkerPoint {
            point: [x as f32, y as f32, z as f32],
            cameras,
            residual: residual as i8 as f32,
        }
    }
}

fn byte_to_bool(byte: u8) -> [bool; 7] {
    let mut bools = [false; 7];
    for i in 0..7 {
        bools[i] = byte & (1 << i) != 0;
    }
    bools
}
