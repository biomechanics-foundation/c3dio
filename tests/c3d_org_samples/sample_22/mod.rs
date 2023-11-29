use c3dio::prelude::*;

#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

#[test]
fn sample22() {
    // Sample22: 2d data and analog data
    assert_read_write("tests/c3d_org_samples/sample_22/BKINtechnologies.c3d");
}
