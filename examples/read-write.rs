use std::assert_eq;

use c3dio::file_formats::Sto;
use c3dio::file_formats::Trc;
use c3dio::prelude::*;

fn main() {
    let c3d = C3d::load("tests/c3d_org_samples/sample_30/admarche2.c3d").unwrap();

    //    Sto::from_c3d(&c3d).write("examples/short-copy2.sto").unwrap();
    let c3d = c3d.write("examples/short-copy.c3d").unwrap();
    let c3d2 = C3d::load("examples/short-copy.c3d").unwrap();
    // let c3d2 = C3d::load("tests/c3d_org_samples/sample_01/Eb015pr.c3d").unwrap();
    assert_eq!(c3d, c3d2);
    dbg!(c3d.points.descriptions);
    dbg!(c3d2.points.descriptions);
    // assert_eq!(c3d.points.size(), c3d2.points.size());
    // for i in 0..c3d.points.size().0 {
    //     for j in 0..c3d.points.size().1 {
    //         if c3d.points[i][j] != c3d2.points[i][j] {
    //             dbg!(i, j);
    //             dbg!(c3d.points[i][j]);
    //             dbg!(c3d2.points[i][j]);
    //             dbg!(c3d2.points.scale_factor);
    //         }
    //     }
    // }
}
