use crate::C3dParseError;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum DataFormat {
    #[default]
    Float,
    Integer,
}

pub fn get_point_bytes_per_frame(point_format: &DataFormat, points_per_frame: usize) -> usize {
    let bytes_per_point = match point_format {
        DataFormat::Float => 16,
        DataFormat::Integer => 8,
    };

    (bytes_per_point * points_per_frame) as usize
}

pub fn get_analog_bytes_per_frame(
    point_format: &DataFormat,
    analog_samples_per_frame: u16,
) -> Result<usize, C3dParseError> {
    let bytes_per_analog_point = match point_format {
        DataFormat::Float => 4,
        DataFormat::Integer => 2,
    } as usize;

    Ok(bytes_per_analog_point * analog_samples_per_frame as usize)
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MarkerPoint {
    pub point: [f32; 3],
    pub residual: f32,
    pub cameras: [bool; 7],
}

impl Deref for MarkerPoint {
    type Target = [f32; 3];

    fn deref(&self) -> &Self::Target {
        &self.point
    }
}

impl DerefMut for MarkerPoint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.point
    }
}

//impl Index<usize> for MarkerPoint {
//    type Output = f32;
//
//    fn index(&self, index: usize) -> &Self::Output {
//        match index {
//            0 => &self.x,
//            1 => &self.y,
//            2 => &self.z,
//            _ => panic!("Index out of bounds"),
//        }
//    }
//}
//
//impl IndexMut<usize> for MarkerPoint {
//    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//        match index {
//            0 => &mut self.x,
//            1 => &mut self.y,
//            2 => &mut self.z,
//            _ => panic!("Index out of bounds"),
//        }
//    }
//}

impl MarkerPoint {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            point: [x, y, z],
            residual: 0.0,
            cameras: [false; 7],
        }
    }

    pub fn scale(&mut self, scale: f32) -> Self {
        self.point[0] *= scale;
        self.point[1] *= scale;
        self.point[2] *= scale;
        *self
    }

    pub fn scale_residual(&mut self, scale: f32) -> Self {
        self.residual *= scale;
        *self
    }

    pub(crate) fn cameras_as_byte(&self) -> u8 {
        let mut cameras_byte = 0;
        for (i, camera) in self.cameras.iter().enumerate() {
            if *camera {
                cameras_byte |= 1 << i;
            }
        }
        cameras_byte
    }
}
