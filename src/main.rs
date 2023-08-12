use c3dio::C3d;
fn main() {
    let sample19 = C3d::load("tests/c3d.org-sample-files/Sample00/Vicon Motion Systems/TableTennis.c3d").unwrap();

    dbg!(sample19.data.points[(0, 0)]);
    dbg!(sample19.data.points[(1, 0)]);
    dbg!(sample19.data.points[(0, 1)]);

}
