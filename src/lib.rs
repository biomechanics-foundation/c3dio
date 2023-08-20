//! # c3dio - Pure Rust C3D Parser
//! This crate provides a parser for C3D files.
//! C3D is a binary file format used to store motion capture data.
//! The format is described in the [C3D file format documentation](https://www.c3d.org/HTML/default.htm).
//!
//! # Examples
//! ```
//! use c3d::prelude::*;
//!
//! let c3d = C3d::load("tests/data/short.c3d");
//! assert!(c3d.is_ok());
//! ```

use std::path::PathBuf;
use std::{error::Error, fmt};

#[path = "bytes.rs"]
mod bytes;
#[path = "c3d.rs"]
pub mod c3d;
#[path = "data.rs"]
pub mod data;
#[path = "events.rs"]
pub mod events;
#[path = "parameters.rs"]
pub mod parameters;
#[path = "processor.rs"]
mod processor;

use bytes::Bytes;
use data::Data;
use events::Events;
use parameters::Parameters;
use processor::Processor;

pub mod prelude {
    pub use crate::{parameters::ParameterData, C3d, C3dParseError};
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

/// Represents a parsed C3D file.
/// The file can be read from disk or from memory.
///
/// # Examples
///
/// ```
/// use c3d::prelude::*;
/// use std::path::PathBuf;
/// let c3d = C3d::load(PathBuf::from("tests/data/short.c3d"));
/// assert!(c3d.is_ok());
/// ```
#[derive(Debug)]
pub struct C3d {
    pub file_path: Option<PathBuf>,
    bytes: Bytes,
    pub parameters: Parameters,
    processor: Processor,
    pub data: Data,
    pub events: Events,
}
