/// This test set is complete.

use c3dio::prelude::*;

#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

// Sample03: human gait data

#[test]
fn gait_raw() {
    assert_read_write("tests/c3d_org_samples/sample_03/gait-raw.c3d");
}

#[test]
fn gait_pig() {
    assert_read_write("tests/c3d_org_samples/sample_03/gait-pig.c3d");
}

#[test]
fn gait_pig_nz() {
    assert_read_write("tests/c3d_org_samples/sample_03/gait-pig-nz.c3d");
}
