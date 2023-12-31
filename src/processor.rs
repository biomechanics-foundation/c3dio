//! Internal module for converting bytes to the correct format based on the processor type.
use crate::C3dParseError;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum Processor {
    Dec,
    Intel,
    SgiMips,
    #[default]
    Unknown,
}

impl ToString for Processor {
    fn to_string(&self) -> String {
        match self {
            Processor::Intel => "Intel",
            Processor::Dec => "DEC",
            Processor::SgiMips => "SGI MIPS",
            Processor::Unknown => "Unknown",
        }
        .to_string()
    }
}


/// Processor is used to conveniently calculate the value of specific bytes
/// based on the processor type. C3D files can be created on different
/// processors and the bytes are stored differently based on the processor.
/// The processor type is stored in the parameter start block.
impl Processor {
    #[allow(dead_code)]
    pub(crate) fn new() -> Processor {
        Processor::default()
    }

    /// Convenience function to create a Processor from the parameter start block.
    pub(crate) fn from_parameter_start_block(
        parameter_start_block: [u8; 512],
    ) -> Result<Processor, C3dParseError> {
        match parameter_start_block[3] {
            0x54 => Ok(Processor::Intel),
            0x55 => Ok(Processor::Dec),
            0x56 => Ok(Processor::SgiMips),
            _ => Err(C3dParseError::InvalidProcessorType),
        }
    }

    /// Calculates the u16 value from the bytes based on the processor type.
    pub(crate) fn u16(self, bytes: [u8; 2]) -> u16 {
        match self {
            Processor::Intel => intel_u16(bytes),
            Processor::Dec => dec_u16(bytes),
            Processor::SgiMips => sgi_mips_u16(bytes),
            Processor::Unknown => 0,
        }
    }

    /// Calculates the i16 value from the bytes based on the processor type.
    pub(crate) fn i16(self, bytes: [u8; 2]) -> i16 {
        match self {
            Processor::Intel => intel_i16(bytes) as i16,
            Processor::Dec => dec_i16(bytes) as i16,
            Processor::SgiMips => sgi_mips_i16(bytes) as i16,
            Processor::Unknown => 0,
        }
    }

    /// Calculates the f32 value from the bytes based on the processor type.
    pub(crate) fn f32(self, bytes: [u8; 4]) -> f32 {
        match self {
            Processor::Intel => intel_f32(bytes),
            Processor::Dec => dec_f32(bytes),
            Processor::SgiMips => sgi_mips_f32(bytes),
            Processor::Unknown => 0.0,
        }
    }

    /// Calculates the bytes from the u16 value based on the processor type.
    pub(crate) fn u16_to_bytes(self, value: u16) -> [u8; 2] {
        match self {
            Processor::Intel => value.to_le_bytes(),
            Processor::Dec => value.to_le_bytes(),
            Processor::SgiMips => value.to_be_bytes(),
            Processor::Unknown => [0, 0],
        }
    }

    pub(crate) fn i16_to_bytes(self, value: i16) -> [u8; 2] {
        match self {
            Processor::Intel => value.to_le_bytes(),
            Processor::Dec => value.to_le_bytes(),
            Processor::SgiMips => value.to_be_bytes(),
            Processor::Unknown => [0, 0],
        }
    }

    /// Calculates the bytes from the f32 value based on the processor type.
    pub(crate) fn f32_to_bytes(self, value: f32) -> [u8; 4] {
        match self {
            Processor::Intel => value.to_le_bytes(),
            Processor::Dec => {
                let temp = value.to_le_bytes();
                if temp[3] == 255 {
                    [temp[2], temp[3], temp[0], temp[1]]
                } else {
                    [temp[2], temp[3] + 1, temp[0], temp[1]]
                }
            }
            Processor::SgiMips => value.to_be_bytes(),
            Processor::Unknown => [0, 0, 0, 0],
        }
    }
}

/// Conversion of the raw bytes into intel u16 format
fn intel_u16(bytes: [u8; 2]) -> u16 {
    u16::from_le_bytes(bytes)
}

/// Conversion of the raw bytes into dec u16 format
fn dec_u16(bytes: [u8; 2]) -> u16 {
    u16::from_le_bytes(bytes)
}

/// Conversion of the raw bytes into sgi_mips u16 format
fn sgi_mips_u16(bytes: [u8; 2]) -> u16 {
    u16::from_be_bytes(bytes)
}

/// Conversion of the raw bytes into intel i16 format
fn intel_i16(bytes: [u8; 2]) -> i16 {
    i16::from_le_bytes(bytes)
}

/// Conversion of the raw bytes into dec i16 format
fn dec_i16(bytes: [u8; 2]) -> i16 {
    i16::from_le_bytes(bytes)
}

/// Conversion of the raw bytes into sgi_mips i16 format
fn sgi_mips_i16(bytes: [u8; 2]) -> i16 {
    i16::from_be_bytes(bytes)
}

/// Conversion of the raw bytes into intel f32 format
fn intel_f32(bytes: [u8; 4]) -> f32 {
    f32::from_le_bytes(bytes)
}

/// Conversion of the raw bytes into dec f32 format based on the following:
/// https://stackoverflow.com/questions/64760137/how-to-display-dec-floating-point-format-given-32-bits-in-ieee-standard
fn dec_f32(bytes: [u8; 4]) -> f32 {
    if bytes.len() != 4 {
        return 0.0;
    }
    if bytes[1] == 0x00 {
        let bytes = [bytes[2], bytes[3], bytes[0], bytes[1]];
        f32::from_le_bytes(bytes)
    } else {
        let bytes = [bytes[2], bytes[3], bytes[0], bytes[1] - 1];
        f32::from_le_bytes(bytes)
    }
}

/// Conversion of the raw bytes into sgi_mips f32 format
fn sgi_mips_f32(bytes: [u8; 4]) -> f32 {
    f32::from_be_bytes(bytes)
}
