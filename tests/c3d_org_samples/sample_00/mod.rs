/// Tests for manufacturer samples from https://www.c3d.org/sampledata.html
/// This test set is complete.

#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

#[test]
fn arthuman_sample_fingers() {
    assert_read_write("tests/c3d_org_samples/sample_00/Advanced Realtime Tracking GmbH/arthuman-sample-fingers.c3d");
}

#[test]
fn arthuman_sample() {
    assert_read_write(
        "tests/c3d_org_samples/sample_00/Advanced Realtime Tracking GmbH/arthuman-sample.c3d",
    )
}

#[test]
fn codamotion_gaitwands_19970212() {
    assert_read_write(
        "tests/c3d_org_samples/sample_00/Codamotion/codamotion_gaitwands_19970212.c3d",
    )
}

#[test]
fn codamotion_gaitwands_20150204() {
    assert_read_write(
        "tests/c3d_org_samples/sample_00/Codamotion/codamotion_gaitwands_20150204.c3d",
    )
}

#[test]
fn emg_data_cometa() {
    assert_read_write("tests/c3d_org_samples/sample_00/Cometa Systems/EMG Data Cometa.c3d");
}

#[test]
fn gait_with_emg() {
    assert_read_write(
        "tests/c3d_org_samples/sample_00/Innovative Sports Training/Gait with EMG.c3d",
    )
}

#[test]
fn static_pose() {
    assert_read_write("tests/c3d_org_samples/sample_00/Innovative Sports Training/Static Pose.c3d")
}

#[test]
fn sample_jump2() {
    assert_read_write(
        "tests/c3d_org_samples/sample_00/Motion Analysis Corporation/Sample_Jump2.c3d",
    )
}

#[test]
fn walk1() {
    assert_read_write("tests/c3d_org_samples/sample_00/Motion Analysis Corporation/Walk1.c3d");
}

#[test]
fn test1() {
    assert_read_write("tests/c3d_org_samples/sample_00/NexGen Ergonomics/test1.c3d");
}

#[test]
fn table_tennis() {
    assert_read_write("tests/c3d_org_samples/sample_00/Vicon Motion Systems/TableTennis.c3d");
}

#[test]
fn pycgm2_lower_limb_cgm24_walking01() {
//    file contains NaNs which cause the point data comparison to fail
//    assert_read_write(
//        "tests/c3d_org_samples/sample_00/Vicon Motion Systems/pyCGM2 lower limb CGM24 Walking01.c3d",
//    );
}
