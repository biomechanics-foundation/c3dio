use c3dio::prelude::*;

#[test]
fn sample15() {
    // Sample15: missing parameter errors
    assert!(C3d::load("tests/c3d_org_samples/sample_15/FP1.c3d").is_err());
    assert!(C3d::load("tests/c3d_org_samples/sample_15/FP2.c3d").is_err());
}
