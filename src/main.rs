mod data;
mod header;
mod parameters;
mod parse;
mod processor;

use std::env;

use crate::parameters::read_parameters_from_file;
use crate::parse::C3dParseError;

fn main() -> Result<(), C3dParseError> {
    env::set_var("RUST_BACKTRACE", "1");
    let args: Vec<String> = env::args().collect();
    dbg!(&args);

    let parameters = read_parameters_from_file("res/Sample01/Eb015si.c3d");
    match parameters {
        Ok(parameters) => {
            dbg!(parameters);
        }
        Err(e) => {
            dbg!(e);
        }
    }
    //    let now = std::time::Instant::now();
    //
    //    for _ in 0..500 {
    //        read_header_from_file(&args[1])?;
    //    }
    //
    //    println!("Time elapsed: {}ms", now.elapsed().as_millis());

    Ok(())
}
