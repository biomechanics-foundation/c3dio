#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

#[test]
fn sample21() {
    // Sample21: incorrect POINT:RATE and ANALOG:RATE
    // TODO: figure out why this doesn't fail
    assert_read_write("tests/c3d_org_samples/sample_21/sample21.c3d");
}
