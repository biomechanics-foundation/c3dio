/// This test set is complete.

use c3dio::prelude::*;

#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

#[test]
fn sample06() {
    // Sample06: parameter name errors
    let mac_walk = C3d::load("tests/c3d_org_samples/sample_06/MACsample.c3d");
    let new_walk = C3d::load("tests/c3d_org_samples/sample_06/newwalk.c3d");

    assert!(mac_walk.is_err());
    assert!(new_walk.is_ok());

    assert_read_write("tests/c3d_org_samples/sample_06/newwalk.c3d");

}
