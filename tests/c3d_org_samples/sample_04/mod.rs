/// This test set is complete.

#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

// Sample04: human gait data with parameters to identify two subjects

#[test]
fn bodybuilder() {
    assert_read_write("tests/c3d_org_samples/sample_04/bodybuilder.c3d");
}

#[test]
fn sub_labels() {
    assert_read_write("tests/c3d_org_samples/sample_04/sub_labels.c3d");
}
