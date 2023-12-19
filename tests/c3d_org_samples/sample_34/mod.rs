#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

// Three sample C3D data files exported from the MEMS based Xsens motion capture system containing 3D data
// only and includes the TRIAL group with start and end field entries defing the data collection frames.
// The files also include an unnessesary Group named TERMINATOR which has the description "Terminates the
// groups and parameters section."
//
// 05/01/2012  02:32 PM         1,579,008 AlbertoINT.c3d
// 05/01/2012  02:32 PM         2,525,184 AlbertoREAL.c3d
// 05/01/2012  02:32 PM           401,920 Basketball.c3d
//
// AlbertoINT.c3d 	- Integer C3D file.
// AlbertoREAL.c3d - REAL or floating point file.
// Basketball.c3d	- sample data file.

#[test]
fn alberto_int() {
    assert_read_write("tests/c3d_org_samples/sample_34/AlbertoINT.c3d");
}

#[test]
fn alberto_real() {
    assert_read_write("tests/c3d_org_samples/sample_34/AlbertoREAL.c3d");
}

#[test]
fn basketball() {
    assert_read_write("tests/c3d_org_samples/sample_34/Basketball.c3d");
}