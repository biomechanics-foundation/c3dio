#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

#[test]
fn sample20() {
    // Sample20: missing parameters
    // TODO: investigate why this doesn't fail
    assert_read_write("tests/c3d_org_samples/sample_20/phasespace_sample.c3d");
}
