use c3dio::prelude::*;

#[test]
fn sample18() {
    // Sample18: corrupted parameter section
    assert!(C3d::load("tests/c3d_org_samples/sample_18/bad_parameter_section.c3d").is_err());
}
