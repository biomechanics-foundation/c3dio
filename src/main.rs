
use std::env;

use silico_c3d::prelude::*;
//use std::fs::File;

fn main() -> Result<(), C3dParseError> {
        env::set_var("RUST_BACKTRACE", "1");
        let args: Vec<String> = env::args().collect();
        dbg!(&args);

        let contents = read_c3d("res/Sample00/Vicon Motion Systems/TableTennis.c3d")?;

        let c3d = C3d::from_bytes(contents.as_slice())?;

        dbg!(&c3d.data.point_data.data[(0, 0, 0)]);
        dbg!(&c3d.data.point_data.data[(0, 0, 1)]);
        dbg!(&c3d.data.point_data.data[(0, 0, 2)]);
        dbg!(&c3d.data.point_data.data[(0, 1, 0)]);
        dbg!(&c3d.data.point_data.data[(0, 1, 1)]);
        dbg!(&c3d.data.point_data.data[(0, 1, 2)]);

//    let mut file = match File::open("foo.txt") {
//        Ok(file) => file,
//        Err(e) => return Err(C3dParseError::ReadError(e)),
//    };
//    let mut buf = [0; 10];
//    let header = match file.read_exact(&mut buf) {
//        Ok(_) => buf,
//        Err(e) => return Err(C3dParseError::ReadError(e)),
//    };

    Ok(())
}
