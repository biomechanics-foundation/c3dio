#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

#[test]
fn sample27() {
    // Sample27: Kyowa Dengyo force plate data stored as TYPE-3
    assert_read_write("tests/c3d_org_samples/sample_27/kyowadengyo.c3d");
}
