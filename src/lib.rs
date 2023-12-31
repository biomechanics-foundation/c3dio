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

pub use analog::Analog;
pub use analog::AnalogFormat;
pub use analog::AnalogOffset;
pub use c3d::C3d;
pub use data::DataFormat;
pub use data::MarkerPoint;
pub use events::Event;
pub use events::EventContext;
pub use events::Events;
pub use forces::ForcePlatform;
pub use forces::ForcePlatformCorners;
pub use forces::ForcePlatformOrigin;
pub use forces::ForcePlatformType;
pub use forces::ForcePlatforms;
pub use manufacturer::Manufacturer;
pub use manufacturer::ManufacturerVersion;
pub use parameters::{Parameter, ParameterData, Parameters};
pub use points::Points;
pub use processor::Processor;
pub use processor::ProcessorType;
pub use seg::Seg;
pub use file_formats::trc::Trc;
pub use file_formats::sto::Sto;

/// Contains the most commonly used types and functions from this crate.
pub mod prelude {
    pub use crate::{
        Analog, AnalogFormat, AnalogOffset, C3d, C3dParseError, C3dWriteError, Events,
        ForcePlatform, ForcePlatformType, ForcePlatforms, Manufacturer, ManufacturerVersion,
        MarkerPoint, Parameter, ParameterData, Parameters, Points, Processor, ProcessorType, Seg, Sto, Trc
    };
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

/// Reports errors that occurred while writing a C3D file.
/// The error type is returned by the `write` method.
#[derive(Debug)]
pub enum C3dWriteError {
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

impl Error for C3dWriteError {}
impl fmt::Display for C3dWriteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "C3dWriteError: {:?}", self)
    }
}
