use crate::parameters::Parameters;
use crate::parse::C3dParseError;
use crate::processor::{bytes_to_f32, bytes_to_i16, ProcessorType};
use ndarray::{Array, Array2, Array3};

#[derive(Debug)]
pub enum DataFormat {
    Float,
    Integer,
}

#[derive(Debug)]
pub struct Data {
    pub point_data: PointData,
    analog_data: AnalogData,
    data_format: DataFormat,
}

#[derive(Debug)]
pub struct PointData {
    pub data: Array3<f32>,
    cameras: Array3<bool>,
    residual: Array2<f32>,
    pub labels: Vec<String>,
}

#[derive(Debug)]
pub struct AnalogData {
    data: Array2<f32>,
    labels: Vec<String>,
}

pub fn parse_data(
    data_blocks: &[u8],
    parameters: &Parameters,
    processor_type: &ProcessorType,
) -> Result<Data, C3dParseError> {
    let (data_format, num_frames, num_points, point_labels, point_scale, point_rate) =
        get_point_info(&parameters)?;
    let (num_analog_channels, analog_labels, analog_sample_rate, analog_scale) =
        get_analog_info(&parameters)?;

    let analog_points_per_frame = (analog_sample_rate / point_rate).round() as usize;

    let bytes_per_point = match data_format {
        DataFormat::Float => 16,
        DataFormat::Integer => 8,
    };

    let point_bytes_per_frame = bytes_per_point * num_points;

    let bytes_per_analog_point = match data_format {
        DataFormat::Float => 4,
        DataFormat::Integer => 2,
    };

    let analog_bytes_per_frame =
        bytes_per_analog_point * num_analog_channels * analog_points_per_frame;

    let bytes_per_frame = point_bytes_per_frame + analog_bytes_per_frame;

    let mut point_data: Array3<f32> = Array::zeros((num_frames, num_points, 3));
    let mut analog_data: Array2<f32> = Array::zeros((num_frames, num_analog_channels));
    let mut cameras: Array3<bool> = Array::from_elem((num_frames, num_points, 7), false);
    let mut residual: Array2<f32> = Array::zeros((num_frames, num_points));

    let mut point_iter = point_data.iter_mut();
    let mut analog_iter = analog_data.iter_mut();
    let mut camera_iter = cameras.iter_mut();
    let mut residual_iter = residual.iter_mut();
    for i in 0..num_frames {
        let start = i * bytes_per_frame;
        let end = start + bytes_per_frame;
        let frame = &data_blocks[start..end];
        let point_frame_data = &frame[0..point_bytes_per_frame];
        let analog_frame_data = &frame[point_bytes_per_frame..];
        for j in 0..num_points {
            let start = j * bytes_per_point;
            let end = start + bytes_per_point;
            let point_slice = &point_frame_data[start..end];
            let (x, y, z, cameras, residual) = match data_format {
                DataFormat::Float => parse_point_data_float(point_slice, processor_type),
                DataFormat::Integer => parse_point_data_int(point_frame_data, processor_type),
            };
            *point_iter.next().unwrap() = x;
            *point_iter.next().unwrap() = y;
            *point_iter.next().unwrap() = z;
            for k in 0..7 {
                *camera_iter.next().unwrap() = cameras[k];
            }
            *residual_iter.next().unwrap() = residual;
        }
        for j in 0..analog_points_per_frame {
            let start = j * bytes_per_analog_point * num_analog_channels;
            let end = start + bytes_per_analog_point * num_analog_channels;
            let analog_slice = &analog_frame_data[start..end];
            let analog_data = match data_format {
                DataFormat::Float => {
                    parse_analog_data_float(analog_slice, num_analog_channels, processor_type)
                }
                DataFormat::Integer => {
                    parse_analog_data_int(analog_slice, num_analog_channels, processor_type)
                }
            };
        }
    }

    Ok(Data {
        point_data: PointData {
            data: point_data,
            cameras,
            residual,
            labels: point_labels,
        },
        analog_data: AnalogData {
            data: analog_data,
            labels: analog_labels,
        },
        data_format,
    })
}

fn parse_point_data_float(
    point_frame_data: &[u8],
    processor_type: &ProcessorType,
) -> (f32, f32, f32, [bool; 7], f32) {
    let x = bytes_to_f32(&point_frame_data[0..4], processor_type);
    let y = bytes_to_f32(&point_frame_data[4..8], processor_type);
    let z = bytes_to_f32(&point_frame_data[8..12], processor_type);
    let cameras = i16_to_bool(bytes_to_i16(&point_frame_data[12..14], processor_type));
    let residual = bytes_to_i16(&point_frame_data[14..16], processor_type) as f32;
    (x, y, z, cameras, residual)
}

fn parse_point_data_int(
    point_frame_data: &[u8],
    processor_type: &ProcessorType,
) -> (f32, f32, f32, [bool; 7], f32) {
    let x = bytes_to_i16(&point_frame_data[0..2], processor_type) as f32;
    let y = bytes_to_i16(&point_frame_data[2..4], processor_type) as f32;
    let z = bytes_to_i16(&point_frame_data[4..6], processor_type) as f32;
    let cameras = byte_to_bool(point_frame_data[6]);
    let residual = point_frame_data[7] as f32;
    (x, y, z, cameras, residual)
}

fn byte_to_bool(byte: u8) -> [bool; 7] {
    let mut bools = [false; 7];
    for i in 8..1 {
        bools[i] = byte & (1 << i) != 0;
    }
    bools
}

fn i16_to_bool(i16: i16) -> [bool; 7] {
    let mut bools = [false; 7];
    for i in 16..9 {
        bools[i] = i16 & (1 << i) != 0;
    }
    bools
}

fn parse_analog_data_float(
    analog_frame_data: &[u8],
    num_analog_channels: usize,
    processor_type: &ProcessorType,
) -> Vec<f32> {
    let mut analog_data = Vec::with_capacity(num_analog_channels);
    for i in 0..num_analog_channels {
        let start = i * 4;
        let end = start + 4;
        let analog_slice = &analog_frame_data[start..end];
        let analog = bytes_to_f32(analog_slice, processor_type);
        analog_data.push(analog);
    }
    analog_data
}

fn parse_analog_data_int(
    analog_frame_data: &[u8],
    num_analog_channels: usize,
    processor_type: &ProcessorType,
) -> Vec<f32> {
    let mut analog_data = Vec::with_capacity(num_analog_channels);
    for i in 0..num_analog_channels {
        let start = i * 2;
        let end = start + 2;
        let analog_slice = &analog_frame_data[start..end];
        let analog = bytes_to_i16(analog_slice, processor_type) as f32;
        analog_data.push(analog);
    }
    analog_data
}

fn get_point_info(
    parameters: &Parameters,
) -> Result<(DataFormat, usize, usize, Vec<String>, f32, f32), C3dParseError> {
    let data_format = parameters.get_data_format();
    let num_frames = parameters.get_num_frames();
    let num_points = parameters.get_num_points();
    let point_labels = parameters.get_point_labels();
    let point_scale = parameters.get_point_scale();
    let point_rate = parameters.get_point_rate();
    if let (
        Some(data_format),
        Some(num_frames),
        Some(num_points),
        Some(point_labels),
        Some(point_scale),
        Some(point_rate),
    ) = (
        data_format,
        num_frames,
        num_points,
        point_labels,
        point_scale,
        point_rate,
    ) {
        Ok((
            data_format,
            num_frames,
            num_points,
            point_labels,
            point_scale,
            point_rate,
        ))
    } else {
        Err(C3dParseError::MissingParameter(
            "Missing parameter for point data".to_string(),
        ))
    }
}

fn get_analog_info(
    parameters: &Parameters,
) -> Result<(usize, Vec<String>, f32, f32), C3dParseError> {
    let num_analog_channels = parameters.get_num_analog_channels();
    let analog_labels = parameters.get_analog_labels();
    let analog_sample_rate = parameters.get_analog_sample_rate();
    let analog_scale = parameters.get_analog_scale();
    if let (
        Some(num_analog_channels),
        Some(analog_labels),
        Some(analog_sample_rate),
        Some(analog_scale),
    ) = (
        num_analog_channels,
        analog_labels,
        analog_sample_rate,
        analog_scale,
    ) {
        Ok((
            num_analog_channels,
            analog_labels,
            analog_sample_rate,
            analog_scale,
        ))
    } else {
        Err(C3dParseError::MissingParameter(
            "Missing parameter for analog data".to_string(),
        ))
    }
}
