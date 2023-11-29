use c3dio::prelude::*;

fn main() {
    let c3d = C3d::new();
    c3d.write("examples/simple.c3d").unwrap();
}
