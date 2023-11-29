/// This test set is complete.

use c3dio::prelude::*;

#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

#[test]
fn sample08() {
    // Sample08: check if test files read correct pointer values from the c3d header in block 1
    let pi = C3d::load("tests/c3d_org_samples/sample_08/EB015PI.c3d").unwrap();
    let test_a = C3d::load("tests/c3d_org_samples/sample_08/TESTAPI.c3d").unwrap();
    let test_b = C3d::load("tests/c3d_org_samples/sample_08/TESTBPI.c3d").unwrap();
    let test_c = C3d::load("tests/c3d_org_samples/sample_08/TESTCPI.c3d").unwrap();
    let test_d = C3d::load("tests/c3d_org_samples/sample_08/TESTDPI.c3d").unwrap();

    assert_eq!(pi, test_a);
    assert_eq!(pi, test_b);
    assert_eq!(pi, test_c);
    assert_eq!(pi, test_d);

    assert_read_write("tests/c3d_org_samples/sample_08/EB015PI.c3d");
    assert_read_write("tests/c3d_org_samples/sample_08/TESTAPI.c3d");
    assert_read_write("tests/c3d_org_samples/sample_08/TESTBPI.c3d");
    assert_read_write("tests/c3d_org_samples/sample_08/TESTCPI.c3d");
    assert_read_write("tests/c3d_org_samples/sample_08/TESTDPI.c3d");
}
