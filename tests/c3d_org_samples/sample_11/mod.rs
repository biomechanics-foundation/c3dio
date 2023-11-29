use c3dio::prelude::*;

#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

#[test]
fn poor_1() {
    assert_read_write("tests/c3d_org_samples/sample_11/2198928.c3d");
}

#[test]
fn poor_2() {
    //contains type 3 force plates but force plate channels are only 6 wide instead of 8
    //assert_read_write("tests/c3d_org_samples/sample_11/evart.c3d");
}
