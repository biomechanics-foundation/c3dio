/// This test set is complete.

use c3dio::prelude::*;

#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

#[test]
fn all_partial_eq() {
    // Sample01: each of the 6 files should load to the exact same headers, parameters, and frames
    let pi = C3d::load("tests/c3d_org_samples/sample_01/Eb015pi.c3d").unwrap();
    let pr = C3d::load("tests/c3d_org_samples/sample_01/Eb015pr.c3d").unwrap();
    let si = C3d::load("tests/c3d_org_samples/sample_01/Eb015si.c3d").unwrap();
    let sr = C3d::load("tests/c3d_org_samples/sample_01/Eb015sr.c3d").unwrap();
    let vi = C3d::load("tests/c3d_org_samples/sample_01/Eb015vi.c3d").unwrap();
    let vr = C3d::load("tests/c3d_org_samples/sample_01/Eb015vr.c3d").unwrap();
    assert_eq!(pi, pr);
    assert_eq!(pi, si);
    assert_eq!(pi, sr);
    assert_eq!(pi, vi);
    assert_eq!(pi, vr);
}

#[test]
fn read_write_eb015pi() {
    assert_read_write("tests/c3d_org_samples/sample_01/Eb015pi.c3d");
}

#[test]
fn read_write_eb015pr() {
    assert_read_write("tests/c3d_org_samples/sample_01/Eb015pr.c3d");
}

#[test]
fn read_write_eb015si() {
    assert_read_write("tests/c3d_org_samples/sample_01/Eb015si.c3d");
}

#[test]
fn read_write_eb015sr() {
    assert_read_write("tests/c3d_org_samples/sample_01/Eb015sr.c3d");
}

#[test]
fn read_write_eb015vi() {
    assert_read_write("tests/c3d_org_samples/sample_01/Eb015vi.c3d");
}

#[test]
fn read_write_eb015vr() {
    assert_read_write("tests/c3d_org_samples/sample_01/Eb015vr.c3d");
}
