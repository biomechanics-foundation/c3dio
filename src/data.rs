use crate::processor::Processor;

pub struct Data {
}

#[derive(Debug)]
pub enum DataFormat {
    Float,
    Integer,
    Unknown,
}

pub fn parse_point_data_float(
    point_frame_data: &[u8],
    processor: &Processor,
) -> (f32, f32, f32, [bool; 7], f32) {
    println!("float");
    println!("processor_type: {:?}", processor.processor_type);
    println!("x bytes: {:?}", &point_frame_data[0..4]);
    let x = processor.f32(point_frame_data[0..4].try_into().unwrap());
    let y = processor.f32(point_frame_data[4..8].try_into().unwrap());
    let z = processor.f32(point_frame_data[8..12].try_into().unwrap());
    let cameras = i16_to_bool(processor.i16(point_frame_data[12..14].try_into().unwrap()));
    let residual = processor.i16(point_frame_data[14..16].try_into().unwrap()) as f32;
    (x, y, z, cameras, residual)
}

pub fn parse_point_data_int(
    point_frame_data: &[u8],
    processor: &Processor,
) -> (f32, f32, f32, [bool; 7], f32) {
    println!("int");
    println!("processor_type: {:?}", processor.processor_type);
    println!("x bytes: {:?}", &point_frame_data[0..2]);
    let x = processor.i16(point_frame_data[0..2].try_into().unwrap()) as f32;
    let y = processor.i16(point_frame_data[2..4].try_into().unwrap()) as f32;
    let z = processor.i16(point_frame_data[4..6].try_into().unwrap()) as f32;
    let cameras = byte_to_bool(point_frame_data[6]);
    let residual = point_frame_data[7] as f32;
    (x, y, z, cameras, residual)
}

fn byte_to_bool(byte: u8) -> [bool; 7] {
    let mut bools = [false; 7];
    for i in 8..1 {
        bools[i] = byte & (1 << i) != 0;
    }
    bools
}

fn i16_to_bool(i16: i16) -> [bool; 7] {
    let mut bools = [false; 7];
    for i in 16..9 {
        bools[i] = i16 & (1 << i) != 0;
    }
    bools
}

pub fn parse_analog_data_float(
    analog_frame_data: &[u8],
    num_analog_channels: usize,
    processor: &Processor,
) -> Vec<f32> {
    let mut analog_data = Vec::with_capacity(num_analog_channels);
    for i in 0..num_analog_channels {
        let start = i * 4;
        let end = start + 4;
        let analog_slice = analog_frame_data[start..end].try_into().unwrap();
        let analog = processor.f32(analog_slice);
        analog_data.push(analog);
    }
    analog_data
}

pub fn parse_analog_data_int(
    analog_frame_data: &[u8],
    num_analog_channels: usize,
    processor: &Processor,
) -> Vec<f32> {
    let mut analog_data = Vec::with_capacity(num_analog_channels);
    for i in 0..num_analog_channels {
        let start = i * 2;
        let end = start + 2;
        let analog_slice = analog_frame_data[start..end].try_into().unwrap();
        let analog = processor.i16(analog_slice) as f32;
        analog_data.push(analog);
    }
    analog_data
}

