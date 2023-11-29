use c3dio::prelude::*;

#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

#[test]
fn compare_force_plate_types() {
    // Sample10: differentiate between TYPE-2 and TYPE-4 force data
    // as well as TYPE-3 data with different force plate types
    let two = C3d::load("tests/c3d_org_samples/sample_10/TYPE-2.C3D").unwrap();
    let _twoa = C3d::load("tests/c3d_org_samples/sample_10/TYPE-2a.c3d").unwrap();
    let _three = C3d::load("tests/c3d_org_samples/sample_10/TYPE-3.c3d").unwrap();
    let four = C3d::load("tests/c3d_org_samples/sample_10/TYPE-4.C3D").unwrap();
    let _foura = C3d::load("tests/c3d_org_samples/sample_10/TYPE-4a.c3d").unwrap();

    //TODO: show two and four have the same analog data after the calibration matrix is applied
    //assert_eq!(two.force(0, 0).unwrap(), four.force(0, 0).unwrap());
}

#[test]
fn type_2() {
    assert_read_write("tests/c3d_org_samples/sample_10/TYPE-2.C3D");
}

#[test]
fn type_2a() {
    assert_read_write("tests/c3d_org_samples/sample_10/TYPE-2a.c3d");
}

#[test]
fn type_3() {
    assert_read_write("tests/c3d_org_samples/sample_10/TYPE-3.c3d");
}

#[test]
fn type_4() {
    assert_read_write("tests/c3d_org_samples/sample_10/TYPE-4.C3D");
}

#[test]
fn type_4a() {
    assert_read_write("tests/c3d_org_samples/sample_10/TYPE-4a.c3d");
}
