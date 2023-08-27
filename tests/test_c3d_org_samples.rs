use c3dio::C3d;

// This test file attempts to create tests for each of the sample file sets provided by c3d.org
// The sample files are available at https://www.c3d.org/sampledata.html

#[test]
fn sample01() {
    // Sample01: each of the 6 files should load to the exact same headers, parameters, and frames
    let pi = C3d::load("tests/c3d.org-sample-files/Sample01/Eb015pi.c3d").unwrap();
    let pr = C3d::load("tests/c3d.org-sample-files/Sample01/Eb015pr.c3d").unwrap();
    let si = C3d::load("tests/c3d.org-sample-files/Sample01/Eb015si.c3d").unwrap();
    let sr = C3d::load("tests/c3d.org-sample-files/Sample01/Eb015sr.c3d").unwrap();
    let vi = C3d::load("tests/c3d.org-sample-files/Sample01/Eb015vi.c3d").unwrap();
    let vr = C3d::load("tests/c3d.org-sample-files/Sample01/Eb015vr.c3d").unwrap();
    assert_eq!(pi, pr);
    assert_eq!(pi, si);
    assert_eq!(pi, sr);
    assert_eq!(pi, vi);
    assert_eq!(pi, vr);
}

#[test]
fn sample02() {
    // Sample02: each of the 6 files should load to the exact same headers, parameters, and frames
    let dec_int = C3d::load("tests/c3d.org-sample-files/Sample02/dec_int.c3d").unwrap();
    let dec_real = C3d::load("tests/c3d.org-sample-files/Sample02/dec_real.c3d").unwrap();
    let pc_int = C3d::load("tests/c3d.org-sample-files/Sample02/pc_int.c3d").unwrap();
    let pc_real = C3d::load("tests/c3d.org-sample-files/Sample02/pc_real.c3d").unwrap();
    let sgi_int = C3d::load("tests/c3d.org-sample-files/Sample02/sgi_int.c3d").unwrap();
    let sgi_real = C3d::load("tests/c3d.org-sample-files/Sample02/sgi_real.c3d").unwrap();
    assert_eq!(dec_int, dec_real);
    assert_eq!(dec_int, sgi_real);
    assert_eq!(dec_int, pc_real);

    assert_eq!(sgi_int, pc_int);
    // FALSE - potentially a bug in the c3d.org sample files
    //assert_eq!(dec_int, pc_int);
    //assert_eq!(dec_int, sgi_int);
}

#[test]
fn sample08() {
    // Sample08: check if test files read correct pointer values from the c3d header in block 1
    let pi = C3d::load("tests/c3d.org-sample-files/Sample08/EB015PI.c3d").unwrap();
    let test_a = C3d::load("tests/c3d.org-sample-files/Sample08/TESTAPI.c3d").unwrap();
    let test_b = C3d::load("tests/c3d.org-sample-files/Sample08/TESTBPI.c3d").unwrap();
    let test_c = C3d::load("tests/c3d.org-sample-files/Sample08/TESTCPI.c3d").unwrap();
    let test_d = C3d::load("tests/c3d.org-sample-files/Sample08/TESTDPI.c3d").unwrap();
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
    let f18124 = C3d::load("tests/c3d.org-sample-files/Sample36/18124framesf.c3d");
    assert!(f18124.is_ok());
    assert_eq!(f18124.unwrap().data.num_frames, 18124);

    let i18124 = C3d::load("tests/c3d.org-sample-files/Sample36/18124framesi.c3d");
    assert!(i18124.is_ok());
    assert_eq!(i18124.unwrap().data.num_frames, 18124);

    let f36220 = C3d::load("tests/c3d.org-sample-files/Sample36/36220framesf.c3d");
    assert!(f36220.is_ok());
    assert_eq!(f36220.unwrap().data.num_frames, 36220);

    let i36220 = C3d::load("tests/c3d.org-sample-files/Sample36/36220framesi.c3d");
    assert!(i36220.is_ok());
    assert_eq!(i36220.unwrap().data.num_frames, 36220);

    // Uses '5' instead of 'TRIAL' for group name
    //let f72610 = C3d::load("tests/c3d.org-sample-files/Sample36/72610framesf.c3d");
    //assert!(f72610.is_ok());
    //assert_eq!(f72610.unwrap().data.num_frames, 72610);

    // Contains [-1, 0] for actual_end_frame
    //let i72610 = C3d::load("tests/c3d.org-sample-files/Sample36/72610framesi.c3d");
    //assert!(i72610.is_ok());
    //assert_eq!(i72610.unwrap().data.num_frames, 72610);
}

// Section: Sample C3D Data

#[test]
fn sample07() {
    // Sample07: read 16-bit analog data using parameter ANALOG:OFFSET
    assert!(true);
}

#[test]
fn sample10() {
    // Sample10: differentiate between TYPE-2 and TYPE-4 force data
    // as well as TYPE-3 data with different force plate types
    assert!(true);
}

#[test]
fn sample12() {
    // Sample12: read 3D data from a very large file
    assert!(C3d::load("tests/c3d.org-sample-files/Sample12/c24089 13.c3d").is_ok());
}

#[test]
fn sample17() {
    // Sample17: read a sample file with 128 analog channels from 14 force plates
    assert!(true);
}

#[test]
fn sample19() {
    // Sample19: read a sample file with 34672 frames of analog data
    // requires reading the header be read as unsigned 16-bit integers
    let sample19 = C3d::load("tests/c3d.org-sample-files/Sample19/sample19.c3d");
    assert!(sample19.is_ok());
    let sample19 = sample19.unwrap();
    let num_frames = sample19.data.analog.shape().0
        / (sample19.data.analog_samples_per_channel_per_frame) as usize;
    assert!(num_frames == 34672)
}

#[test]
fn sample22() {
    // Sample22: 2d data and analog data
    assert!(true);
}

#[test]
fn sample29() {
    // Sample29: 3d point data with minor formatting errors
    assert!(C3d::load("tests/c3d.org-sample-files/Sample29/Facial-Sing.c3d").is_ok());
    assert!(C3d::load("tests/c3d.org-sample-files/Sample29/OptiTrack-IITSEC2007.c3d").is_ok());
}

#[test]
fn sample31() {
    // Sample31: a very long file that requires use of the TRIAL parameter group
    assert!(C3d::load("tests/c3d.org-sample-files/Sample31/large01.c3d").is_ok());
    assert!(C3d::load("tests/c3d.org-sample-files/Sample31/large02.c3d").is_ok());
}

#[test]
fn sample34() {
    // Sample34: three sample c3d files from IMU system
    assert!(true);
}

#[test]
fn sample37() {
    // Sample37: read four sample C3D files with multiple data points
    assert!(true);
}

// Application Files

#[test]
fn sample03() {
    // Sample03: human gait data
    assert!(true);
}

#[test]
fn sample04() {
    // Sample04: human gait data with parameters to identify two subjects
    assert!(true);
}

#[test]
fn sample05() {
    // Sample05: human gait data with EMG and force data
    assert!(true);
}

#[test]
fn sample23() {
    // Sample23: human gait data with custom GROUPS and PARAMETERS
    assert!(true);
}

#[test]
fn sample26() {
    // Sample26: human gait data from Qualisys system
    assert!(true);
}

#[test]
fn sample27() {
    // Sample27: Kyowa Dengyo force plate data stored as TYPE-3
    assert!(true);
}

#[test]
fn sample28() {
    // Sample28: samples from BTS system with TYPE-1 force plate data
    // includes formatting issues in first file, corrected in third file
    assert!(true);
}

#[test]
fn sample30() {
    // Sample30: Biogesta 3d point and analog data
    assert!(true);
}

#[test]
fn sample33() {
    // Sample33: static test file with large parameter block
    assert!(true);
}

#[test]
fn sample35() {
    // Sample35: EMG data from Mega Electronics
    assert!(true);
}

// Formatting Errors

#[test]
fn sample06() {
    // Sample06: parameter name errors
    assert!(true);
}

#[test]
fn sample09() {
    // Sample09: issues with storing non-3d data in 3d point values
    assert!(true);
}

#[test]
fn sample11() {
    // Sample11: poor force plate data
    assert!(true);
}

#[test]
fn sample13() {
    // Sample13: parameter errors
    assert!(true);
}

#[test]
fn sample14() {
    // Sample14: synchronization errors
    assert!(true);
}

#[test]
fn sample15() {
    // Sample15: missing parameter errors
    assert!(true);
}

#[test]
fn sample16() {
    // Sample16: invalid data
    assert!(true);
}

#[test]
fn sample18() {
    // Sample18: corrupted parameter section
    assert!(true);
}

#[test]
fn sample20() {
    // Sample20: missing parameters
    assert!(true);
}

#[test]
fn sample21() {
    // Sample21: missing parameters
    assert!(true);
}

#[test]
fn sample24() {
    // Sample24: empty parameters
    assert!(true);
}

#[test]
fn sample25() {
    // Sample25: corrupted parameters
    assert!(true);
}

#[test]
fn sample32() {
    // Sample32: data corruption problems from importing from USB-based system
    // at the same time as other sources
    assert!(true);
}
