use c3dio::parameters::AnalogOffset::{Signed, Unsigned};
use std::ops::{MulAssign, SubAssign};

use c3dio::C3d;
fn main() {
    let sample19 = C3d::load("tests/c3d.org-sample-files/Sample19/sample19.c3d").unwrap();

    dbg!(sample19.data.num_frames);
    dbg!(sample19.data.analog_channels);
    dbg!(sample19.data.analog_samples_per_frame);
    dbg!(
        (sample19.data.analog.dim().0)
            / (sample19.data.analog_samples_per_frame / sample19.data.analog_channels) as usize
    );
}
