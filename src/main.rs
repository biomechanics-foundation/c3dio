use c3dio::C3d;
fn main() {
    let pi = C3d::load("tests/c3d.org-sample-files/Sample01/Eb015pi.c3d").unwrap();
    let pr = C3d::load("tests/c3d.org-sample-files/Sample01/Eb015pr.c3d").unwrap();
}
