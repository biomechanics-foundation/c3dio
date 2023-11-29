use c3dio::prelude::*;

#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

#[test]
fn sample25() {
    // Sample25: corrupted parameters
    assert_read_write("tests/c3d_org_samples/sample_25/analogfpscale01.c3d");
    assert_read_write("tests/c3d_org_samples/sample_25/analogfpscale02.c3d");
    assert_read_write("tests/c3d_org_samples/sample_25/analogfpscale03.c3d");
    assert_read_write("tests/c3d_org_samples/sample_25/analogfpscale04.c3d");
    assert_read_write("tests/c3d_org_samples/sample_25/analogfpscale04i.c3d");
}
