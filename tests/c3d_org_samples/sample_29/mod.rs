use c3dio::prelude::*;

#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

#[test]
fn sample29() {
    // Sample29: 3d point data with minor formatting errors
    assert_read_write("tests/c3d_org_samples/sample_29/Facial-Sing.c3d");
    assert_read_write("tests/c3d_org_samples/sample_29/OptiTrack-IITSEC2007.c3d");
}
