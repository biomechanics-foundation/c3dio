use std::assert_eq;

use c3dio::prelude::*;

fn main() {
    let c3d = C3d::load("tests/c3d_org_samples/sample_30/admarche2.c3d").unwrap();

    let c3d = c3d.write("examples/short-copy.c3d").unwrap();
    let c3d2 = C3d::load("examples/short-copy.c3d").unwrap();
    assert_eq!(c3d, &c3d2);
}
