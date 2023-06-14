use ndarray::{Array2, Array3};
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

#[path = "c3d.rs"]
pub mod c3d;
#[path = "events.rs"]
pub mod events;
#[path = "data.rs"]
pub mod data;
#[path = "parameters.rs"]
pub mod parameters;
#[path = "processor.rs"]
pub mod processor;

use data::DataFormat;
use parameters::ParameterData;
use processor::Processor;
use events::Events;

#[derive(Debug)]
pub enum C3dParseError {
    ReadError(std::io::Error),
    InsufficientBlocks(String),
    InvalidHeaderStartBlock,
    InvalidParameterStartBlock,
    InvalidParameterData,
    InvalidDataStartBlock,
    InvalidProcessorType,
    InvalidDataType,
    InvalidParametersOffset,
    MissingGroup(String),
    MissingParameter(String),
    UnknownDataFormat,
    InvalidGroupId,
    MissingPointScale,
    FileNotOpen,
    NotEnoughData,
    InvalidNextParameter,
    TooManyEvents(usize),
}

#[derive(Debug)]
pub struct C3d {
    pub file_path: PathBuf,
    pub file: Option<File>,
    pub header_bytes: [u8; 512],
    pub parameter_bytes: Vec<u8>,
    pub parameters: HashMap<String, HashMap<String, (ParameterData, String)>>,
    pub group_descriptions: HashMap<String, String>,
    pub parameter_start_block_index: usize,
    pub num_frames: usize,
    pub points: Array3<f32>,
    pub point_labels: Vec<String>,
    pub cameras: Array3<bool>,
    pub residuals: Array2<f32>,
    pub analog: Array2<f32>,
    pub analog_labels: Vec<String>,
    pub analog_channels: u16,
    pub analog_samples_per_frame: u16,
    pub data_format: DataFormat,
    pub data_bytes: Vec<u8>,
    pub data_start_block_index: usize,
    pub processor: Processor,
    pub points_per_frame: u16,
    pub first_frame: u16,
    pub last_frame: u16,
    pub max_interpolation_gap: u16,
    pub scale_factor: f32,
    pub frame_rate: f32,
    pub events: Events,
}
