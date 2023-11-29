#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

#[test]
fn capture0002() {
    assert_read_write("tests/c3d_org_samples/sample_26/Capture0002.c3d");
}

#[test]
fn capture0003() {
    assert_read_write("tests/c3d_org_samples/sample_26/Capture0003.c3d");
}

#[test]
fn capture0004() {
    assert_read_write("tests/c3d_org_samples/sample_26/Capture0004.c3d");
}

#[test]
fn capture0008_standing() {
    assert_read_write("tests/c3d_org_samples/sample_26/Capture0008_Standing.c3d");
}

#[test]
fn standing_hybrid_1() {
    assert_read_write("tests/c3d_org_samples/sample_26/Standing_Hybrid_1.c3d");
}

#[test]
fn standing_hybrid_2() {
    assert_read_write("tests/c3d_org_samples/sample_26/Standing_Hybrid_2.c3d");
}

#[test]
fn walking_hybrid_1_1() {
    assert_read_write("tests/c3d_org_samples/sample_26/Walking_Hybrid_1_1.c3d");
}

#[test]
fn walking_hybrid_1_2() {
    assert_read_write("tests/c3d_org_samples/sample_26/Walking_Hybrid_1_2.c3d");
}

#[test]
fn walking_hybrid_1_3() {
    assert_read_write("tests/c3d_org_samples/sample_26/Walking_Hybrid_1_3.c3d");
}

#[test]
fn walking_hybrid_1_4() {
    assert_read_write("tests/c3d_org_samples/sample_26/Walking_Hybrid_1_4.c3d");
}

#[test]
fn walking_hybrid_1_5() {
    assert_read_write("tests/c3d_org_samples/sample_26/Walking_Hybrid_1_5.c3d");
}

#[test]
fn walking_hybrid_2() {
    assert_read_write("tests/c3d_org_samples/sample_26/Walking_Hybrid_2.c3d");
}
