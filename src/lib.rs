use std::path::PathBuf;
use std::{error::Error, fmt};

#[path = "bytes.rs"]
pub mod bytes;
#[path = "c3d.rs"]
pub mod c3d;
#[path = "data.rs"]
pub mod data;
#[path = "events.rs"]
pub mod events;
#[path = "parameters.rs"]
pub mod parameters;
#[path = "processor.rs"]
pub mod processor;

use bytes::Bytes;
use data::Data;
use events::Events;
use parameters::Parameters;
use processor::Processor;

pub mod prelude {
    pub use crate::{parameters::ParameterData, C3d, C3dParseError};
}

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
    GroupNotFound(String),
    ParameterNotFound(String, String),
    RequiredParameterNotFound(String),
    InvalidData(String),
    InvalidParameterFormat(String),
}

impl Error for C3dParseError {}
impl fmt::Display for C3dParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "C3dParseError: {:?}", self)
    }
}

#[derive(Debug)]
pub struct C3d {
    pub file_path: Option<PathBuf>,
    pub bytes: Bytes,
    pub parameters: Parameters,
    pub processor: Processor,
    pub data: Data,
    pub events: Events,
}
