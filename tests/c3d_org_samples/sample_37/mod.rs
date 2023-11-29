/// This test set is complete.

#[path = "../../common.rs"]
mod common;
use common::assert_read_write;

// Sample37: read four sample C3D files with multiple data points

#[test]
fn two_persons_integer() {
    // ART-Human_2persons_integer.c3d : 2 persons; integer coordinates, 3 points per joint
    assert_read_write("tests/c3d_org_samples/sample_37/ART-Human_2persons_integer.c3d");
}

#[test]
fn hands_real_no6dof() {
    // ART-Human_hands_real_no6dof.c3d : 1 person with fingers, floating point coordinates, 1 point per joint
    assert_read_write("tests/c3d_org_samples/sample_37/ART-Human_hands_real_no6dof.c3d");
}

#[test]
fn real() {
    // ART-Human_real.c3d : 1 person; floating point coordinates, 3 points per joint
    assert_read_write("tests/c3d_org_samples/sample_37/ART-Human_real.c3d");
}

#[test]
fn tools_integer_6dof_limbs() {
    // ART-Human_tools_integer_6dofLimbs.c3d : 1 person, 2 tools; integer coordinates, 1 point per joint, 3 points per limb
    assert_read_write("tests/c3d_org_samples/sample_37/ART-Human_tools_integer_6dofLimbs.c3d");
}
