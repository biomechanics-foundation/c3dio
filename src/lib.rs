//! # c3dio - Pure Rust C3D Parser
//! This crate provides a parser for C3D files.
//! C3D is a binary file format used to store motion capture data.
//! The format is described in the [C3D file format documentation](https://www.c3d.org/HTML/default.htm).
//!
//! # Examples
//! ```
//! use c3dio::prelude::*;
//!
//! let c3d = C3d::load("tests/data/short.c3d");
//! assert!(c3d.is_ok());
//! ```

use std::path::PathBuf;
use std::{error::Error, fmt};

#[path = "analog.rs"]
pub mod analog;
#[path = "c3d.rs"]
pub mod c3d;
#[path = "data.rs"]
pub mod data;
#[path = "events.rs"]
pub mod events;
#[path = "forces.rs"]
pub mod forces;
#[path = "manufacturer.rs"]
pub mod manufacturer;
#[path = "parameters.rs"]
pub mod parameters;
#[path = "points.rs"]
pub mod points;
#[path = "processor.rs"]
mod processor;
#[path = "seg.rs"]
pub mod seg;

#[path = "file_formats/mod.rs"]
pub mod file_formats;

use analog::Analog;
use events::Events;
use forces::ForcePlatforms;
use manufacturer::Manufacturer;
use parameters::{Parameter, Parameters};
use points::Points;
use processor::Processor;
use seg::Seg;

pub mod prelude {
    pub use crate::{parameters::ParameterData, C3d, C3dIoError, C3dParseError};
}

/// Reports errors that occurred while parsing a C3D file.
/// The error type is returned by the `load` and `from_bytes` methods.
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
    InvalidDescription,
    MissingGroup(String),
    MissingParameter(String),
    InvalidGroupId,
    MissingPointScale,
    FileNotOpen,
    NotEnoughData,
    InvalidNextParameter,
    TooManyEvents(i16),
    NumFramesMismatch(usize, usize),
    GroupNotFound(String),
    ParameterNotFound(String, String),
    RequiredParameterNotFound(String),
    InvalidData(Parameter, String),
    InvalidParameterFormat(String),
    AnalogOffsetScaleMismatch,
    InsufficientAnalogOffsets,
    InvalidParameterDimensions(String),
    InvalidParameterType(String),
    InvalidEventLabel(String, String),
    MissingEventTime(usize),
    MissingEventLabel(usize),
    NoParameterTimeEvents,
    HeaderNotParsed,
    AnalogBytesPerFrameMismatch,
    FrameRateMismatch(f32, f32),
    ScaleFactorMismatch(f32, f32),
}

impl Error for C3dParseError {}
impl fmt::Display for C3dParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "C3dParseError: {:?}", self)
    }
}

#[derive(Debug)]
pub enum C3dIoError {
    WriteError(PathBuf, std::io::Error),
    InvalidFileExtension(String),
    InvalidFilePath(PathBuf),
    WriteHeaderError(std::io::Error),
    WriteParametersError(std::io::Error),
    WriteDataError(std::io::Error),
    GroupNameTooLong(String),
    GroupNameNotAscii(String),
    GroupDescriptionTooLong(String),
    ParameterNameTooLong(String),
    ParameterNameNotAscii(String),
    InvalidParameterDimensions(String),
    ParameterDescriptionTooLong(String),
    InvalidForcePlatformInfo(String),
}

impl Error for C3dIoError {}
impl fmt::Display for C3dIoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "C3dIoError: {:?}", self)
    }
}

/// Represents a parsed C3D file.
/// The file can be read from disk or from memory.
///
/// # Examples
///
/// ```
/// use c3dio::prelude::*;
///
/// let c3d = C3d::load("tests/data/short.c3d");
/// assert!(c3d.is_ok());
/// ```
pub struct C3d {
    pub parameters: Parameters,
    processor: Processor,
    pub points: Points,
    pub analog: Analog,
    pub events: Events,
    pub manufacturer: Manufacturer,
    pub seg: Seg,
    pub forces: ForcePlatforms,
    header_bytes: [u8; 512],
}
