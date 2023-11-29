use c3dio::prelude::*;

#[test]
fn sample19() {
    // Sample19: read a sample file with 34672 frames of analog data
    // requires reading the header be read as unsigned 16-bit integers
    let sample19 = C3d::load("tests/c3d_org_samples/sample_19/sample19.c3d");
    assert!(sample19.is_ok());
    let sample19 = sample19.unwrap();
    let num_frames = sample19.analog.analog.rows()
        / (sample19.analog.samples_per_channel_per_frame) as usize;
    assert!(num_frames == 34672)
}
