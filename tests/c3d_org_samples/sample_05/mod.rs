/// This test set is complete.

use c3dio::prelude::*;

#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

#[test]
fn sample05() {
    // Sample05: human gait data with EMG and force data
    let vicon = C3d::load("tests/c3d_org_samples/sample_05/vicon512.c3d").unwrap();

    assert_eq!(vicon.points.size().0, 6492);

    assert_read_write("tests/c3d_org_samples/sample_05/vicon512.c3d");
}
