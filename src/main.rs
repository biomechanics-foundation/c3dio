pub mod data;
pub mod header;
pub mod parameters;
pub mod parse;
pub mod processor;
pub mod c3d;

use std::env;

use crate::parse::{read_c3d, C3dParseError};
use crate::c3d::C3d;

fn main() -> Result<(), C3dParseError> {
    env::set_var("RUST_BACKTRACE", "1");
    let args: Vec<String> = env::args().collect();
    dbg!(&args);

    let contents = read_c3d("res/Sample01/Eb015si.c3d")?;

    dbg!(C3d::from_bytes(contents.as_slice())?);


    Ok(())
}

