/// This test set is complete.

use c3dio::prelude::*;

#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

#[test]
fn very_large_c3d_file() {
    // Sample12: read 3D data from a very large file
    // shortened to stay below 100MB GitHub file size limit
    let large = C3d::load("tests/c3d_org_samples/sample_12/c24089 13.c3d").unwrap();
    assert_eq!(large.points.size().0, 17922);
    assert_eq!(large.points.size().1, 364);
}

#[test]
fn c23089_13() {
    //contains a 3D point with a NaN value
    //assert_read_write("tests/c3d_org_samples/sample_12/c24089 13.c3d");
}
