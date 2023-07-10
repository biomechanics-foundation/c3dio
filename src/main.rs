use c3dio::parameters::AnalogOffset::{Signed, Unsigned};
use std::ops::{MulAssign, SubAssign};

use c3dio::C3d;
fn main() {
    let dec_int = C3d::load("tests/c3d.org-sample-files/Sample19/sample19.c3d").unwrap();
}
