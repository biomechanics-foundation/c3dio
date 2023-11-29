use c3dio::analog::AnalogOffset;
use c3dio::prelude::*;

#[test]
fn sixteen_bit_analog() {
    // Sample07: read 16-bit analog data using parameter ANALOG:OFFSET
    let c3d = C3d::load("tests/c3d_org_samples/sample_07/16bitanalog.c3d").unwrap();
    match c3d.analog.offset {
        AnalogOffset::Signed(offset) => {
            assert_eq!(offset[0], 32767);
        }
        _ => assert!(false),
    };
}
