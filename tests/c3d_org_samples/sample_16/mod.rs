use c3dio::prelude::*;

// Sample16: invalid data

#[test]
fn basketball() {
    assert!(C3d::load("tests/c3d_org_samples/sample_16/basketball.c3d").is_err());
}

#[test]
fn giant() {
    assert!(C3d::load("tests/c3d_org_samples/sample_16/giant.c3d").is_err());
}

#[test]
fn golf() {
    assert!(C3d::load("tests/c3d_org_samples/sample_16/golf.c3d").is_err());
}


