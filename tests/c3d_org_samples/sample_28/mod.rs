use c3dio::prelude::*;

#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

use crate::c3d_org_samples::sample_28::common::assert_eq_data;

// Sample28: samples from BTS system with TYPE-1 force plate data
// includes formatting issues in first file, corrected in third file

#[test]
fn dynamic() {
    assert!(C3d::load("tests/c3d_org_samples/sample_28/dynamic.C3D").is_err());
}

#[test]
fn standing() {
    assert!(C3d::load("tests/c3d_org_samples/sample_28/standing.C3D").is_err());
}

#[test]
fn type1() {
    // does not include an ANALOG:RATE parameter
    //assert_read_write("tests/c3d_org_samples/sample_28/type1.C3D");
}
