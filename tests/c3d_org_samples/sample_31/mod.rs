
#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

// Sample31: a very long file that requires use of the TRIAL parameter group

#[test]
fn large01() {
    assert_read_write("tests/c3d_org_samples/sample_31/large01.c3d");
}

#[test]
fn large02() {
    //contains NaNs in the point data
    //assert_read_write("tests/c3d_org_samples/sample_31/large02.c3d");
}
