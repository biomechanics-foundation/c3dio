use std::collections::HashSet;

use c3dio::data::DataFormat;
use c3dio::data::{parse_point_data_float, parse_point_data_int};
use c3dio::C3d;
fn main() {
    let dec_int = C3d::load("tests/c3d.org-sample-files/Sample13/golfswing1.c3d").unwrap();
    //    step_through_data_bytes(pi, pr);
}

fn step_through_data_bytes(c3d1: C3d, c3d2: C3d) {
    let bytes_per_frame1 = c3d1.data.point_bytes_per_frame + c3d1.data.analog_bytes_per_frame;
    let bytes_per_point1 = c3d1.data.point_bytes_per_frame / c3d1.data.points_per_frame as usize;
    let bytes_per_frame2 = c3d2.data.point_bytes_per_frame + c3d2.data.analog_bytes_per_frame;
    let bytes_per_point2 = c3d2.data.point_bytes_per_frame / c3d2.data.points_per_frame as usize;
    let mut counter = 0;
    let mut top_byte_values: HashSet<u8> = HashSet::new();
    let mut bottom_byte_values: HashSet<u8> = HashSet::new();
    //for i in 0..c3d1.data.num_frames {
    for i in 0..1 {
        let start1 = i * bytes_per_frame1;
        let end1 = start1 + bytes_per_frame1;
        let start2 = i * bytes_per_frame2;
        let end2 = start2 + bytes_per_frame2;
        let frame1 = &c3d1.data_bytes[start1..end1];
        let frame2 = &c3d2.data_bytes[start2..end2];
        //for j in 0..c3d1.data.points_per_frame as usize {
        for j in 0..5 {
            let start1 = j * bytes_per_point1;
            let end1 = start1 + bytes_per_point1;
            let start2 = j * bytes_per_point2;
            let end2 = start2 + bytes_per_point2;
            let point1 = &frame1[start1..end1];
            let point2 = &frame2[start2..end2];
            let (mut x1, mut y1, mut z1, _, _) = match c3d1.data.format {
                DataFormat::Float => parse_point_data_float(point1, &c3d1.processor),
                DataFormat::Integer => parse_point_data_int(point1, &c3d1.processor),
                DataFormat::Unknown => panic!("Unknown data format"),
            };
            let (mut x2, mut y2, mut z2, _, _) = match c3d2.data.format {
                DataFormat::Float => parse_point_data_float(point2, &c3d2.processor),
                DataFormat::Integer => parse_point_data_int(point2, &c3d2.processor),
                DataFormat::Unknown => panic!("Unknown data format"),
            };
            if c3d1.data.format == DataFormat::Integer {
                x1 *= c3d1.data.scale_factor;
                y1 *= c3d1.data.scale_factor;
                z1 *= c3d1.data.scale_factor;
            }
            if c3d2.data.format == DataFormat::Integer {
                x2 *= c3d2.data.scale_factor;
                y2 *= c3d2.data.scale_factor;
                z2 *= c3d2.data.scale_factor;
            }
            println!("Frame: {}, Point: {}", i, j);
            println!("x1: {}, x2: {}", x1, x2);
            println!("y1: {}, y2: {}", y1, y2);
            println!("z1: {}, z2: {}", z1, z2);
            if x1 != x2 {
                dbg!("Point mismatch");
                dbg!("point1");
                print_bits(point1[0]);
                print_bits(point1[1]);
                dbg!("point2");
                print_bits(point2[0]);
                print_bits(point2[1]);
                dbg!(x1);
                dbg!(x2);
                top_byte_values.insert(point1[0]);
                bottom_byte_values.insert(point1[1]);
                counter += 1;
            }
            if y1 != y2 {
                dbg!("Point mismatch");
                dbg!("point1");
                print_bits(point1[2]);
                print_bits(point1[3]);
                dbg!("point2");
                print_bits(point2[2]);
                print_bits(point2[3]);
                dbg!(y1);
                dbg!(y2);
                counter += 1;
                top_byte_values.insert(point1[2]);
                bottom_byte_values.insert(point1[3]);
            }
            if z1 != z2 {
                dbg!("Point mismatch");
                dbg!("point1");
                print_bits(point1[4]);
                print_bits(point1[5]);
                dbg!("point2");
                print_bits(point2[4]);
                print_bits(point2[5]);
                dbg!(z1);
                dbg!(z2);
                counter += 1;
                top_byte_values.insert(point1[4]);
                bottom_byte_values.insert(point1[5]);
            }
        }
    }
    dbg!(counter);
    dbg!(c3d1.data.scale_factor);
    dbg!(c3d2.data.scale_factor);
    c3d1 == c3d2;
    //    top_byte_values.iter().for_each(|x| print_bits(*x));
    //    dbg!(bottom_byte_values);
}

fn print_bits(byte: u8) {
    println!("{:#010b}", byte);
}

fn print_stuff(pc_real: C3d, pc_int: C3d) {
    dbg!(pc_real == pc_int);

    dbg!(pc_real.data.points.len());
    dbg!(pc_int.data.points.len());
    dbg!((&pc_real.data.points - &pc_int.data.points).sum());
    let diff = &pc_real.data.points - &pc_int.data.points;
    let mut counter = 0;
    let mut indices = Vec::new();
    diff.iter().for_each(|x| {
        if *x != 0.0 {
            dbg!(x);
            indices.push(counter);
        }
        counter += 1;
    });
    pc_real.data.points.iter().enumerate().for_each(|(i, x)| {
        if indices.contains(&i) {
            dbg!(x);
        }
    });
    pc_int.data.points.iter().enumerate().for_each(|(i, x)| {
        if indices.contains(&i) {
            dbg!(x / pc_int.data.scale_factor);
        }
    });
    dbg!(pc_real.data.scale_factor);
    dbg!(pc_int.data.scale_factor);
    dbg!(pc_real == pc_int);
}
