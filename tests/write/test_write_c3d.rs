use c3dio::prelude::*;

#[test]
fn test() {
    let c3d = C3d::load("tests/data/short.c3d").unwrap();
    let c3d = c3d.write("tests/data/short-copy.c3d").unwrap();
    let c3d2 = C3d::load("tests/data/short-copy.c3d").unwrap();
    assert_eq!(c3d, c3d2);
}
