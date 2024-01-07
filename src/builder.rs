//! The builder module is focused on building a C3d struct for writing to a file.
//! It is focused on composing parts into a valid C3d struct.

use crate::c3d::C3d;
use std::{error::Error, fmt};

/// The C3dBuilder is a struct that is used to build a C3d struct.
#[derive(Debug, Default)]
pub struct C3dBuilder {
    pub(crate) c3d: C3d,
}

impl C3dBuilder {
    /// Creates a new C3dBuilder.
    pub fn new() -> Self {
        C3dBuilder {
            c3d: C3d::default(),
        }
    }

    /// Consumes the builder and returns a C3d struct.
    pub fn build(self) -> C3d {
        self.c3d
    }
}

/// Reports errors in building a C3d struct.
#[derive(Debug)]
pub enum C3dBuilderError {
    InvalidParameter,
}

impl Error for C3dBuilderError {}
impl fmt::Display for C3dBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "C3dBuilderError: {:?}", self)
    }
}
