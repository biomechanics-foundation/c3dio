#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

#[test]
fn sample32() {
    // Sample32: data corruption problems from importing from USB-based system
    // at the same time as other sources
    assert_read_write("tests/c3d_org_samples/sample_32/vicon_zerowire.c3d");
}
