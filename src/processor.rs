use crate::C3dParseError;

#[derive(Debug, Copy, Clone)]
pub struct Processor {
    pub processor_type: ProcessorType,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ProcessorType {
    Dec,
    Intel,
    SgiMips,
    Unknown,
}

impl Processor {
    pub fn new() -> Processor {
        Processor {
            processor_type: ProcessorType::Unknown,
        }
    }
    pub fn from_parameter_start_block(
        parameter_start_block: [u8; 512],
    ) -> Result<Processor, C3dParseError> {
        let processor_type = match parameter_start_block[3] {
            0x54 => Ok(ProcessorType::Intel),
            0x55 => Ok(ProcessorType::Dec),
            0x56 => Ok(ProcessorType::SgiMips),
            _ => Err(C3dParseError::InvalidProcessorType),
        }?;
        Ok(Processor {
            processor_type,
        })
    }
    pub fn u16(self, bytes: [u8; 2]) -> u16 {
        match self.processor_type {
            ProcessorType::Intel => intel_u16(bytes),
            ProcessorType::Dec => dec_u16(bytes),
            ProcessorType::SgiMips => sgi_mips_u16(bytes),
            ProcessorType::Unknown => 0,
        }
    }

    pub fn i16(self, bytes: [u8; 2]) -> i16 {
        match self.processor_type {
            ProcessorType::Intel => intel_i16(bytes) as i16,
            ProcessorType::Dec => dec_i16(bytes) as i16,
            ProcessorType::SgiMips => sgi_mips_i16(bytes) as i16,
            ProcessorType::Unknown => 0,
        }
    }

    pub fn f32(self, bytes: [u8; 4]) -> f32 {
        match self.processor_type {
            ProcessorType::Intel => intel_f32(bytes),
            ProcessorType::Dec => dec_f32(bytes),
            ProcessorType::SgiMips => sgi_mips_f32(bytes),
            ProcessorType::Unknown => 0.0,
        }
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
