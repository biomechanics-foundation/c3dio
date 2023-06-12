use c3dio::C3d;

// This test file attempts to create tests for each of the sample file sets provided by c3d.org
// The sample files are available at https://www.c3d.org/sampledata.html

#[test]
fn sample01() {
    // Sample01: each of the 6 files should load to the exact same headers, parameters, and frames
    let pi = C3d::load("c3d.org-sample/files/Sample01/Eb015pi.c3d").unwrap();
    let pr = C3d::load("c3d.org-sample/files/Sample01/Eb015pr.c3d").unwrap();
    let si = C3d::load("c3d.org-sample/files/Sample01/Eb015si.c3d").unwrap();
    let sr = C3d::load("c3d.org-sample/files/Sample01/Eb015sr.c3d").unwrap();
    let vi = C3d::load("c3d.org-sample/files/Sample01/Eb015vi.c3d").unwrap();
    let vr = C3d::load("c3d.org-sample/files/Sample01/Eb015vr.c3d").unwrap();
    assert_eq!(pi, pr);
    assert_eq!(pi, si);
    assert_eq!(pi, sr);
    assert_eq!(pi, vi);
    assert_eq!(pi, vr);
}

#[test]
fn sample02() {
    // Sample02: each of the 6 files should load to the exact same headers, parameters, and frames
    let dec_int = C3d::load("c3d.org-sample/files/Sample02/dec_int.c3d").unwrap();
    let dec_real = C3d::load("c3d.org-sample/files/Sample02/dec_real.c3d").unwrap();
    let pc_int = C3d::load("c3d.org-sample/files/Sample02/pc_int.c3d").unwrap();
    let pc_real = C3d::load("c3d.org-sample/files/Sample02/pc_real.c3d").unwrap();
    let sgi_int = C3d::load("c3d.org-sample/files/Sample02/sgi_int.c3d").unwrap();
    let sgi_real = C3d::load("c3d.org-sample/files/Sample02/sgi_real.c3d").unwrap();
    assert_eq!(dec_int, dec_real);
    assert_eq!(dec_int, pc_int);
    assert_eq!(dec_int, pc_real);
    assert_eq!(dec_int, sgi_int);
    assert_eq!(dec_int, sgi_real);
}

#[test]
fn sample08() {
    // Sample08: check if test files read correct pointer values from the c3d header in block 1
    let pi = C3d::load("c3d.org-sample/files/Sample08/EB015PI.c3d").unwrap();
    let test_a = C3d::load("c3d.org-sample/files/Sample08/TESTAPI.c3d").unwrap();
    let test_b = C3d::load("c3d.org-sample/files/Sample08/TESTBPI.c3d").unwrap();
    let test_c = C3d::load("c3d.org-sample/files/Sample08/TESTCPI.c3d").unwrap();
    let test_d = C3d::load("c3d.org-sample/files/Sample08/TESTDPI.c3d").unwrap();
    assert_eq!(pi, test_a);
    assert_eq!(pi, test_b);
    assert_eq!(pi, test_c);
    assert_eq!(pi, test_d);
}

#[test]
fn sample36() {
    // Sample36: check ability to read C3D files with different frame lengths
    // including using parameter POINT:FRAMES as either integer or floating pt
    // test ability to read only
    assert!(C3d::load("c3d.org-sample/files/Sample36/18124framesf.c3d").is_ok());
    assert!(C3d::load("c3d.org-sample/files/Sample36/18124framesi.c3d").is_ok());
    assert!(C3d::load("c3d.org-sample/files/Sample36/36220framesf.c3d").is_ok());
    assert!(C3d::load("c3d.org-sample/files/Sample36/36220framesi.c3d").is_ok());
    assert!(C3d::load("c3d.org-sample/files/Sample36/72610framesf.c3d").is_ok());
    assert!(C3d::load("c3d.org-sample/files/Sample36/72610framesi.c3d").is_ok());
}

// Section: Sample C3D Data

#[test]
fn sample07() {

}

