#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ProcessorType {
    Dec,
    Intel,
    SgiMips,
}
pub fn intel_u16(bytes: &[u8]) -> u16 {
    u16::from_le_bytes(<[u8; 2]>::try_from(bytes).unwrap_or([0, 0]))
}

pub fn dec_u16(bytes: &[u8]) -> u16 {
    u16::from_le_bytes(<[u8; 2]>::try_from(bytes).unwrap_or([0, 0]))
}

pub fn sgi_mips_u16(bytes: &[u8]) -> u16 {
    u16::from_be_bytes(<[u8; 2]>::try_from(bytes).unwrap_or([0, 0]))
}

pub fn intel_f32(bytes: &[u8]) -> f32 {
    f32::from_le_bytes(<[u8; 4]>::try_from(bytes).unwrap_or([0, 0, 0, 0]))
}

pub fn dec_f32(bytes: &[u8]) -> f32 {
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

pub fn sgi_mips_f32(bytes: &[u8]) -> f32 {
    f32::from_be_bytes(<[u8; 4]>::try_from(bytes).unwrap_or([0, 0, 0, 0]))
}
