use c3dio::prelude::*;
use test_files::TestFiles;

#[allow(dead_code)]
pub fn assert_eq_events(c3d1: &C3d, c3d2: &C3d) {
    assert_eq!(c3d1.events.len(), c3d2.events.len());
    for i in 0..c3d1.events.len() {
        assert_eq!(c3d2.events[i], c3d1.events[i]);
    }
}

#[allow(dead_code)]
pub fn assert_eq_parameters(c3d1: &C3d, c3d2: &C3d) {
    assert_eq!(c3d1.parameters.num_groups(), c3d2.parameters.num_groups());
    for group in c3d1.parameters.groups() {
        assert_eq!(
            c3d2.parameters.parameters(group).unwrap().len(),
            c3d1.parameters.parameters(group).unwrap().len()
        );
    }
}

#[allow(dead_code)]
pub fn assert_eq_data(c3d1: &C3d, c3d2: &C3d) {
    assert_eq!(c3d1.points.size(), c3d2.points.size());
    assert_eq!(c3d1.points, c3d2.points);
    assert_eq!(c3d1.analog.size(), c3d2.analog.size());
    assert_eq!(c3d1.analog, c3d2.analog);
}

#[allow(dead_code)]
pub fn assert_read_write(path: &str) {
    let temp_dir = TestFiles::new();
    temp_dir.file(path, " ");
    let temp_path = temp_dir.path().join(path).to_str().unwrap().to_string();
    let c3d1 = C3d::load(path).unwrap().write(&temp_path).unwrap();
    let c3d2 = C3d::load(&temp_path).unwrap();
    assert_eq!(c3d1, c3d2);
}
