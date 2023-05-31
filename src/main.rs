use std::env;
use silico_c3d::prelude::*;
//use std::fs::File;

fn main() -> Result<(), C3dParseError> {
    env::set_var("RUST_BACKTRACE", "1");
  //  let file1 = C3d::from_file("res/Sample01/Eb015si.c3d")?;
   // println!("{}", file1.points.len());
C3d::from_file("res/Sample01/Eb015si.c3d").unwrap();
    //        let header2 = C3d::header("res/Sample01/Eb015pi.c3d").unwrap();
    //        let header3 = C3d::header("res/Sample01/Eb015vi.c3d").unwrap();
    //        let header4 = C3d::header("res/Sample01/Eb015sr.c3d").unwrap();
    //        let header5 = C3d::header("res/Sample01/Eb015pr.c3d").unwrap();
    //        let header6 = C3d::header("res/Sample01/Eb015vr.c3d").unwrap();
    //        println!("{}", header1.scale_factor);
    //        println!("{}", header2.scale_factor);
    //        println!("{}", header3.scale_factor);
    //        println!("{}", header4.scale_factor);
    //        println!("{}", header5.scale_factor);
    //        println!("{}", header6.scale_factor);
    //        let args: Vec<String> = env::args().collect();
    //        dbg!(&args);
    //
    //        let contents = read_c3d("res/Sample00/Vicon Motion Systems/TableTennis.c3d")?;
    //
    //        let c3d = C3d::from_bytes(contents.as_slice())?;
    //
    //        dbg!(&c3d.data.point_data.data[(0, 0, 0)]);
    //        dbg!(&c3d.data.point_data.data[(0, 0, 1)]);
    //        dbg!(&c3d.data.point_data.data[(0, 0, 2)]);
    //        dbg!(&c3d.data.point_data.data[(0, 1, 0)]);
    //        dbg!(&c3d.data.point_data.data[(0, 1, 1)]);
    //        dbg!(&c3d.data.point_data.data[(0, 1, 2)]);

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
