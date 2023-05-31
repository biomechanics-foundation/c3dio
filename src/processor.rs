use crate::parse::C3dParseError;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ProcessorType {
    Dec,
    Intel,
    SgiMips,
    Unknown,
}

pub fn get_processor_type(first_parameter_block: [u8; 512]) -> Result<ProcessorType, C3dParseError> {
    match first_parameter_block[3] {
        0x54 => Ok(ProcessorType::Intel),
        0x55 => Ok(ProcessorType::Dec),
        0x56 => Ok(ProcessorType::SgiMips),
        _ => Err(C3dParseError::InvalidProcessorType),
    }
}

pub fn bytes_to_u16(bytes: [u8; 2], processor: &ProcessorType) -> u16 {
    match processor {
        ProcessorType::Intel => intel_u16(bytes),
        ProcessorType::Dec => dec_u16(bytes),
        ProcessorType::SgiMips => sgi_mips_u16(bytes),
        ProcessorType::Unknown => 0,
    }
}

pub fn bytes_to_i16(bytes: [u8; 2], processor: &ProcessorType) -> i16 {
    match processor {
        ProcessorType::Intel => intel_i16(bytes) as i16,
        ProcessorType::Dec => dec_i16(bytes) as i16,
        ProcessorType::SgiMips => sgi_mips_i16(bytes) as i16,
        ProcessorType::Unknown => 0,
    }
}

pub fn bytes_to_f32(bytes: [u8; 4], processor: &ProcessorType) -> f32 {
    match processor {
        ProcessorType::Intel => intel_f32(bytes),
        ProcessorType::Dec => dec_f32(bytes),
        ProcessorType::SgiMips => sgi_mips_f32(bytes),
        ProcessorType::Unknown => 0.0,
    }
}

fn intel_u16(bytes: [u8; 2]) -> u16 {
    u16::from_le_bytes(bytes)
}

fn dec_u16(bytes: [u8; 2]) -> u16 {
    u16::from_le_bytes(bytes)
}

fn sgi_mips_u16(bytes: [u8; 2]) -> u16 {
    u16::from_be_bytes(bytes)
}

fn intel_i16(bytes: [u8; 2]) -> i16 {
    i16::from_le_bytes(bytes)
}

fn dec_i16(bytes: [u8; 2]) -> i16 {
    i16::from_le_bytes(bytes)
}

fn sgi_mips_i16(bytes: [u8; 2]) -> i16 {
    i16::from_be_bytes(bytes)
}

fn intel_f32(bytes: [u8; 4]) -> f32 {
    f32::from_le_bytes(bytes)
}

fn dec_f32(bytes: [u8; 4]) -> f32 {
    // https://stackoverflow.com/questions/64760137/how-to-display-dec-floating-point-format-given-32-bits-in-ieee-standard
    if bytes.len() != 4 {
        return 0.0;
    }
    if bytes[1] == 0x00 {
        return 0.0;
    }
    let bytes = [bytes[2], bytes[3], bytes[0], bytes[1] - 1];
    f32::from_le_bytes(bytes)
}

fn sgi_mips_f32(bytes: [u8; 4]) -> f32 {
    f32::from_be_bytes(bytes)
}
