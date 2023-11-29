use c3dio::prelude::*;


#[test]
fn sample24() {
    // Sample24: empty parameters
    assert!(C3d::load("tests/c3d_org_samples/sample_24/MotionMonitorC3D.c3d").is_err());
}
