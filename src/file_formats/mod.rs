//! Structures to represent auxiliary data types that C3D files can be written to.
use crate::data::MarkerPoint;
use grid::Grid;
use std::path::PathBuf;

pub mod trc;

#[derive(Debug, Clone)]
pub struct Trc {
    pub path_file_type: u8,
    pub path_file_type_description: String,
    pub file_name: Option<PathBuf>,
    pub data_rate: f32,
    pub camera_rate: f32,
    pub num_frames: usize,
    pub units: [char; 4],
    pub marker_names: Vec<String>,
    pub first_frame: usize,
    pub data: Grid<MarkerPoint>,
}

pub mod sto;

#[derive(Debug, Clone)]
pub struct Sto {
    pub file_description: Option<String>,
    pub version: u8,
    pub in_degrees: bool,
    pub first_frame: usize,
    pub data_rate: f32,
    pub column_names: Vec<String>,
    pub data: Grid<f64>,
}
