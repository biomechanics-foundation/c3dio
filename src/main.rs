mod data;
mod header;
mod parameters;
mod parse;
mod processor;

use std::env;

use crate::parse::{parse_header_from_file, C3dParseError};

fn main() -> Result<(), C3dParseError> {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);

    let now = std::time::Instant::now();

    for _ in 0..500 {
        parse_header_from_file(&args[1])?;
    }

    println!("Time elapsed: {}ms", now.elapsed().as_millis());

    Ok(())
}
