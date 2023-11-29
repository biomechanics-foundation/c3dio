use c3dio::C3d;
use std::fs::File;
use std::io::{BufReader, Read};

#[test]
fn compare_byte_to_file_load() {
    let sample19_file = C3d::load("tests/data/short.c3d").unwrap();

    let file = File::open("tests/data/short.c3d").unwrap();
    let mut buffer = BufReader::new(file);
    let mut bytes = Vec::new();
    buffer.read_to_end(&mut bytes).unwrap();
    let sample19_bytes = C3d::from_bytes(&bytes).unwrap();

    assert_eq!(sample19_file.parameters, sample19_bytes.parameters);
    assert_eq!(sample19_file.events, sample19_bytes.events);
}

