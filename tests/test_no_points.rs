use c3dio::C3d;

#[test]
fn test_cometa() {
    let result = C3d::load("tests/c3d.org-sample-files/Sample31/large02.c3d");
//    dbg!(&result);
    assert!(result.is_ok());
//    assert!(false);
}
