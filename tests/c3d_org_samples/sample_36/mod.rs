use c3dio::prelude::*;

// Sample36: check ability to read C3D files with different frame lengths
// including using parameter POINT:FRAMES as either integer or floating pt
// test ability to read only

#[test]
fn f18124() {
    let f18124 = C3d::load("tests/c3d_org_samples/sample_36/18124framesf.c3d");
    assert!(f18124.is_ok());
    assert_eq!(f18124.unwrap().points.rows(), 18124);
}

#[test]
fn i18124() {
    let i18124 = C3d::load("tests/c3d_org_samples/sample_36/18124framesi.c3d");
    assert!(i18124.is_ok());
    assert_eq!(i18124.unwrap().points.rows(), 18124);
}

#[test]
fn f36220() {
    let f36220 = C3d::load("tests/c3d_org_samples/sample_36/36220framesf.c3d");
    assert!(f36220.is_ok());
    assert_eq!(f36220.unwrap().points.rows(), 36220);
}

#[test]
fn i36220() {
    let i36220 = C3d::load("tests/c3d_org_samples/sample_36/36220framesi.c3d");
    assert!(i36220.is_ok());
    assert_eq!(i36220.unwrap().points.rows(), 36220);
}

#[test]
fn f72610() {
    //let f72610 = C3d::load("tests/c3d_org_samples/sample_36/72610framesf.c3d");
    //assert!(f72610.is_ok());
    //assert_eq!(f72610.unwrap().points.rows(), 72610);
}

#[test]
fn i72610() {
    // Contains [-1, 0] for actual_end_frame
    //let i72610 = C3d::load("tests/c3d_org_samples/sample_36/72610framesi.c3d");
    //assert!(i72610.is_ok());
    //assert_eq!(i72610.unwrap().points.rows(), 72610);
}
