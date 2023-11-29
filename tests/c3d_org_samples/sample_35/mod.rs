#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

// A sample C3D file containing EMG data created by the Mega Electronics comapny product MegaWin
// containing an isokinetic measurement with two channels of EMG, knee angle and torque.
//
// 03/11/2013  08:54 AM           602,112 Mega Electronics Isokinetic EMG Angle Torque Sample File.c3d
//
// An odd but valid POINT:RATE value of 1 (sample per second) which results in a very high
// ANALOG:RATE value of 1250.  This is unusual but valid.

#[test]
fn mega_electronics() {
    assert_read_write("tests/c3d_org_samples/sample_35/Mega Electronics Isokinetic EMG Angle Torque Sample File.c3d");
}
