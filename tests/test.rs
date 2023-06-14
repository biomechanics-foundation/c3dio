use c3dio::C3d;

#[test]
fn sample01() {
    // Sample08: check if test files read correct pointer values from the c3d header in block 1
    let pi = C3d::load("tests/c3d.org-sample-files/Sample01/Eb015pi.c3d").unwrap();
    let pr = C3d::load("tests/c3d.org-sample-files/Sample01/Eb015pr.c3d").unwrap();
//    let si = C3d::load("tests/c3d.org-sample-files/Sample01/Eb015si.c3d").unwrap();
//    let sr = C3d::load("tests/c3d.org-sample-files/Sample01/Eb015sr.c3d").unwrap();
//    let vi = C3d::load("tests/c3d.org-sample-files/Sample01/Eb015vi.c3d").unwrap();
//    let vr = C3d::load("tests/c3d.org-sample-files/Sample01/Eb015vr.c3d").unwrap();
    assert_eq!(pi, pr);
//    assert_eq!(pi, si);
//    assert_eq!(pi, sr);
//    assert_eq!(pi, vi);
//    assert_eq!(pi, vr);
}
