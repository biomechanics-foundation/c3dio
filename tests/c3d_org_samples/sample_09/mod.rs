/// This test set is complete.

#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

#[test]
fn plug_in_c3d() {
    assert_read_write("tests/c3d_org_samples/sample_09/PlugInC3D.c3d");
}

#[test]
fn polygon_c3d() {
    assert_read_write("tests/c3d_org_samples/sample_09/PolygonC3D.c3d");
}

#[test]
fn ver4_c3d() {
    assert_read_write("tests/c3d_org_samples/sample_09/VER4C3D.c3d");
}
