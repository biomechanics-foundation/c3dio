use crate::processor::{ProcessorType, dec_f32, dec_u16, intel_f32, intel_u16, sgi_mips_f32, sgi_mips_u16};

pub struct Parameters {}

pub fn parse_parameters(parameter_blocks: &Vec<&[u8; 512]>, processor_type: &ProcessorType) -> Parameters {
    parameter_blocks;
    processor_type;
    Parameters {}
}


