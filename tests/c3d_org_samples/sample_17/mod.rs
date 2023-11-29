use c3dio::prelude::*;

#[test]
fn fourteen_force_plates() {
    // Sample17: read a sample file with 128 analog channels from 14 force plates
    let c3d = C3d::load("tests/c3d_org_samples/sample_17/128analogchannels.c3d").unwrap();
    assert!(c3d.analog.cols() == 128);
}
