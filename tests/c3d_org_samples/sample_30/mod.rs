use c3dio::prelude::*;

#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

// Sample30: Biogesta 3d point and analog data

#[test]
fn admarche2() {
    assert_read_write("tests/c3d_org_samples/sample_30/admarche2.c3d");
}

#[test]
fn emgwl() {
    assert_read_write("tests/c3d_org_samples/sample_30/emgwl.c3d");
}

#[test]
fn marche281() {
    assert_read_write("tests/c3d_org_samples/sample_30/marche281.c3d");
}
