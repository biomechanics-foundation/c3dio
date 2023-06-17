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

use data::Data;
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
    NumFramesMismatch(usize, usize),
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
    pub data_bytes: Vec<u8>,
    pub data_start_block_index: usize,
    pub processor: Processor,
    pub data: Data,
    pub events: Events,
}
