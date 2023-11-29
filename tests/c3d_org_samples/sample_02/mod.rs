/// This test set is complete.

use c3dio::prelude::*;

#[path = "../../common.rs"]
mod common;
use common::*;

#[test]
fn eq_parameters() {
    // Sample02: each of the 6 files should load to the exact same headers, parameters, and frames
    let dec_int = C3d::load("tests/c3d_org_samples/sample_02/dec_int.c3d").unwrap();
    let dec_real = C3d::load("tests/c3d_org_samples/sample_02/dec_real.c3d").unwrap();
    let pc_int = C3d::load("tests/c3d_org_samples/sample_02/pc_int.c3d").unwrap();
    let pc_real = C3d::load("tests/c3d_org_samples/sample_02/pc_real.c3d").unwrap();
    let sgi_int = C3d::load("tests/c3d_org_samples/sample_02/sgi_int.c3d").unwrap();
    let sgi_real = C3d::load("tests/c3d_org_samples/sample_02/sgi_real.c3d").unwrap();

    assert_eq_parameters(&dec_int, &dec_real);
    assert_eq_parameters(&dec_int, &sgi_real);
    assert_eq_parameters(&dec_int, &pc_real);
    assert_eq_parameters(&dec_int, &sgi_int);
    assert_eq_parameters(&dec_int, &pc_int);
}

#[test]
fn eq_data() {
    let dec_int = C3d::load("tests/c3d_org_samples/sample_02/dec_int.c3d").unwrap();
    let dec_real = C3d::load("tests/c3d_org_samples/sample_02/dec_real.c3d").unwrap();
    let pc_int = C3d::load("tests/c3d_org_samples/sample_02/pc_int.c3d").unwrap();
    let pc_real = C3d::load("tests/c3d_org_samples/sample_02/pc_real.c3d").unwrap();
    let sgi_int = C3d::load("tests/c3d_org_samples/sample_02/sgi_int.c3d").unwrap();
    let sgi_real = C3d::load("tests/c3d_org_samples/sample_02/sgi_real.c3d").unwrap();

    //TODO: cannot test if data is correct without testing if residual is correct
    //residual is inconsistent between files
    //assert_eq_data(&dec_int, &dec_real);
    //assert_eq_data(&dec_int, &sgi_real);
    //assert_eq_data(&dec_int, &pc_real);
    //assert_eq_data(&dec_int, &sgi_int);
    //assert_eq_data(&dec_int, &pc_int);
}

// TODO: cannot currently test if the headers are equal because the header
// information is integrated into the rest of the data.

#[test]
fn read_write_dec_int() {
    assert_read_write("tests/c3d_org_samples/sample_02/dec_int.c3d");
}

#[test]
fn read_write_dec_real() {
    assert_read_write("tests/c3d_org_samples/sample_02/dec_real.c3d");
}

#[test]
fn read_write_pc_int() {
    assert_read_write("tests/c3d_org_samples/sample_02/pc_int.c3d");
}

#[test]
fn read_write_pc_real() {
    assert_read_write("tests/c3d_org_samples/sample_02/pc_real.c3d");
}

#[test]
fn read_write_sgi_int() {
    assert_read_write("tests/c3d_org_samples/sample_02/sgi_int.c3d");
}

#[test]
fn read_write_sgi_real() {
    assert_read_write("tests/c3d_org_samples/sample_02/sgi_real.c3d");
}

