#[path = "c3d.rs"]
pub mod c3d;

pub mod prelude {
    pub use crate::{
        c3d::C3d,
        parse::C3dParseError,
    };
}
#[path = "data.rs"]
pub mod data;
#[path = "header.rs"]
pub mod header;
#[path = "parameters.rs"]
pub mod parameters;
#[path = "parse.rs"]
pub mod parse;
#[path = "processor.rs"]
pub mod processor;

pub use c3d::*;
