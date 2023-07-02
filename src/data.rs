use crate::processor::Processor;
use crate::C3dParseError;
use ndarray::{Array, Array2, Array3};

#[derive(Debug, Clone)]
pub struct Data {
    pub points: Array3<f32>,
    pub point_labels: Vec<String>,
    pub cameras: Array3<bool>,
    pub residuals: Array2<f32>,
    pub num_frames: usize,
    pub point_frames: usize,
    pub trial_start_frame: usize,
    pub trial_end_frame: usize,
    pub frame_rate: f32,
    pub scale_factor: f32,
    pub max_interpolation_gap: u16,
    pub first_frame: u16,
    pub last_frame: u16,
    pub points_per_frame: u16,
    pub format: DataFormat,
    pub analog: Array2<f32>,
    pub analog_labels: Vec<String>,
    pub analog_channels: u16,
    pub analog_samples_per_frame: u16,
    pub point_bytes_per_frame: usize,
    pub analog_bytes_per_frame: usize,
}

impl PartialEq for Data {
    fn eq(&self, other: &Self) -> bool {
        self.points == other.points
            && self.point_labels == other.point_labels
            && self.cameras == other.cameras
            && self.residuals == other.residuals
            && self.num_frames == other.num_frames
            && self.frame_rate == other.frame_rate
            && self.scale_factor.abs() == other.scale_factor.abs()
            && self.max_interpolation_gap == other.max_interpolation_gap
            && self.first_frame == other.first_frame
            && self.last_frame == other.last_frame
            && self.points_per_frame == other.points_per_frame
            && self.analog == other.analog
            && self.analog_labels == other.analog_labels
            && self.analog_channels == other.analog_channels
            && self.analog_samples_per_frame == other.analog_samples_per_frame
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataFormat {
    Float,
    Integer,
    Unknown,
}

impl Data {
    pub fn new() -> Data {
        Data {
            points: Array3::zeros((0, 0, 0)),
            point_labels: Vec::new(),
            cameras: Array3::from_elem((0, 0, 0), false),
            residuals: Array2::zeros((0, 0)),
            num_frames: 0,
            point_frames: 0,
            trial_start_frame: 0,
            trial_end_frame: 0,
            frame_rate: 0.0,
            scale_factor: 0.0,
            max_interpolation_gap: 0,
            first_frame: 0,
            last_frame: 0,
            points_per_frame: 0,
            format: DataFormat::Unknown,
            analog: Array2::zeros((0, 0)),
            analog_labels: Vec::new(),
            analog_channels: 0,
            analog_samples_per_frame: 0,
            point_bytes_per_frame: 0,
            analog_bytes_per_frame: 0,
        }
    }

    fn calc_num_frames(&mut self, data_bytes: &[u8]) -> Result<&mut Self, C3dParseError> {
        self.num_frames = (self.last_frame as usize - self.first_frame as usize) + 1;
        if self.num_frames != self.point_frames {
            self.num_frames = self.point_frames;
        //    return Err(C3dParseError::NumFramesMismatch(
        //        self.point_frames,
        //        self.num_frames,
        //    ));
        }
        if self.trial_end_frame - self.trial_start_frame + 1 != self.num_frames {
            self.num_frames = self.trial_end_frame - self.trial_start_frame + 1;
        }

        if DataFormat::Unknown == self.format {
            if self.scale_factor < 0.0 {
                self.format = DataFormat::Float;
            } else {
                self.format = DataFormat::Integer;
            }
            //return Err(C3dParseError::UnknownDataFormat);
        }
        let bytes_per_point = match self.format {
            DataFormat::Float => 16,
            DataFormat::Integer => 8,
            DataFormat::Unknown => 0,
        };

        self.point_bytes_per_frame = (bytes_per_point * self.points_per_frame) as usize;

        let bytes_per_analog_point = match self.format {
            DataFormat::Float => 4,
            DataFormat::Integer => 2,
            DataFormat::Unknown => {
                let estimated_analog_bytes = match self.analog_samples_per_frame {
                    0 => 0,
                    _ => {
                        data_bytes.len()
                            / (self.analog_samples_per_frame as usize * self.num_frames)
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

        self.analog_bytes_per_frame =
            bytes_per_analog_point * self.analog_samples_per_frame as usize;

        let bytes_per_frame = self.point_bytes_per_frame + self.analog_bytes_per_frame;
        self.num_frames = match data_bytes.len() < self.num_frames * bytes_per_frame as usize {
            true => {
                let num_frames = data_bytes.len() / bytes_per_frame as usize;
                num_frames
            }
            false => self.num_frames,
            //return Err(C3dParseError::NotEnoughData);
        };
        Ok(self)
    }

    fn parse_points(&mut self, data_bytes: &[u8], processor: &Processor) -> Result<&mut Self, C3dParseError> {
        let mut point_data: Array3<f32> =
            Array::zeros((self.num_frames as usize, self.points_per_frame as usize, 3));
        let mut cameras: Array3<bool> = Array::from_elem(
            (self.num_frames as usize, self.points_per_frame as usize, 7),
            false,
        );
        let mut residual: Array2<f32> =
            Array::zeros((self.num_frames as usize, self.points_per_frame as usize));

        let mut camera_iter = cameras.iter_mut();
        let mut residual_iter = residual.iter_mut();
        let mut point_iter = point_data.iter_mut();

        let bytes_per_frame = self.point_bytes_per_frame + self.analog_bytes_per_frame;
        let bytes_per_point: u16 = match self.points_per_frame {
            0 => 0,
            _ => self.point_bytes_per_frame as u16 / self.points_per_frame as u16,
        };

        for i in 0..self.num_frames {
            let start = i * bytes_per_frame as usize;
            let end = start + bytes_per_frame as usize;
            let point_frame_data = &data_bytes[start..end - self.analog_bytes_per_frame as usize];
            for j in 0..self.points_per_frame {
                let start = j * bytes_per_point;
                let end = start + bytes_per_point;
                let point_slice = &point_frame_data[start as usize..end as usize];
                let (x, y, z, cameras, residual) = match self.format {
                    DataFormat::Float => parse_point_data_float(point_slice, processor),
                    DataFormat::Integer => parse_point_data_int(point_slice, processor),
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
        }

        match self.format {
            DataFormat::Integer => {
                point_data *= self.scale_factor;
                self.points = point_data;
            }
            _ => {
                self.points = point_data;
            }
        }

        Ok(self)
    }

    fn parse_analog(&mut self, data_bytes: &Vec<u8>, processor: &Processor) -> Result<&mut Self, C3dParseError> {
        let mut analog_data: Array2<f32> = Array::zeros((
            self.num_frames as usize,
            self.analog_samples_per_frame as usize,
        ));
        let mut analog_iter = analog_data.iter_mut();

        let analog_samples_per_channel_per_frame = if self.analog_channels > 0 {
            (self.analog_samples_per_frame / self.analog_channels) as usize
        } else {
            0
        };

        let bytes_per_frame = self.point_bytes_per_frame + self.analog_bytes_per_frame;
        let bytes_per_analog_point = match self.analog_samples_per_frame {
            0 => 0,
            _ => self.analog_bytes_per_frame / self.analog_samples_per_frame as usize,
        };
        for i in 0..self.num_frames {
            let start = i * bytes_per_frame as usize;
            let end = start + bytes_per_frame as usize;
            let analog_frame_data = &data_bytes[start + self.point_bytes_per_frame as usize..end];
            for j in 0..self.analog_channels as usize {
                let start = j * bytes_per_analog_point * analog_samples_per_channel_per_frame;
                let end = start + bytes_per_analog_point * analog_samples_per_channel_per_frame;
                let analog_slice = &analog_frame_data[start as usize..end as usize];
                let analog_data = match self.format {
                    DataFormat::Float => parse_analog_data_float(
                        analog_slice,
                        analog_samples_per_channel_per_frame,
                        processor,
                    ),
                    DataFormat::Integer => parse_analog_data_int(
                        analog_slice,
                        analog_samples_per_channel_per_frame,
                        processor,
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
        self.analog = analog_data;
        Ok(self)
    }

    pub fn parse(&mut self, data_bytes: &Vec<u8>, processor: &Processor) -> Result<(), C3dParseError> {
        self.calc_num_frames(data_bytes)?
            .parse_points(data_bytes, processor)?
            .parse_analog(data_bytes, processor)?;
        Ok(())
    }
}

pub fn parse_point_data_float(
    point_frame_data: &[u8],
    processor: &Processor,
) -> (f32, f32, f32, [bool; 7], f32) {
    let x = processor.f32(point_frame_data[0..4].try_into().unwrap());
    let y = processor.f32(point_frame_data[4..8].try_into().unwrap());
    let z = processor.f32(point_frame_data[8..12].try_into().unwrap());
    let cameras = i16_to_bool(processor.i16(point_frame_data[12..14].try_into().unwrap()));
    let residual = processor.i16(point_frame_data[14..16].try_into().unwrap()) as f32;
    (x, y, z, cameras, residual)
}

pub fn parse_point_data_int(
    point_frame_data: &[u8],
    processor: &Processor,
) -> (f32, f32, f32, [bool; 7], f32) {
    let x = processor.i16(point_frame_data[0..2].try_into().unwrap());
    let y = processor.i16(point_frame_data[2..4].try_into().unwrap());
    let z = processor.i16(point_frame_data[4..6].try_into().unwrap());
    let cameras = byte_to_bool(point_frame_data[6]);
    let residual = point_frame_data[7] as f32;
    (x as f32, y as f32, z as f32, cameras, residual)
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

pub fn parse_analog_data_float(
    analog_frame_data: &[u8],
    num_analog_channels: usize,
    processor: &Processor,
) -> Vec<f32> {
    let mut analog_data = Vec::with_capacity(num_analog_channels);
    for i in 0..num_analog_channels {
        let start = i * 4;
        let end = start + 4;
        let analog_slice = analog_frame_data[start..end].try_into().unwrap();
        let analog = processor.f32(analog_slice);
        analog_data.push(analog);
    }
    analog_data
}

pub fn parse_analog_data_int(
    analog_frame_data: &[u8],
    num_analog_channels: usize,
    processor: &Processor,
) -> Vec<f32> {
    let mut analog_data = Vec::with_capacity(num_analog_channels);
    for i in 0..num_analog_channels {
        let start = i * 2;
        let end = start + 2;
        let analog_slice = analog_frame_data[start..end].try_into().unwrap();
        let analog = processor.i16(analog_slice) as f32;
        analog_data.push(analog);
    }
    analog_data
}
