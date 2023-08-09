use c3dio::C3d;
fn main() {
    let sample19 = C3d::load("tests/c3d.org-sample-files/Sample07/16bitanalog.c3d").unwrap();

    dbg!(sample19.data.num_frames);
    dbg!(sample19.data.analog_channels);
    dbg!(sample19.data.analog_samples_per_frame);
    dbg!(sample19.data.analog_samples_per_frame as usize * sample19.data.num_frames);
}
