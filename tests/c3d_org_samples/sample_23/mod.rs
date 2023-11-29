use c3dio::prelude::*;

#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

#[test]
fn sample23() {
    // Sample23: human gait data with custom GROUPS and PARAMETERS
    assert_read_write("tests/c3d_org_samples/sample_23/Vicon_analysis.c3d");
}
