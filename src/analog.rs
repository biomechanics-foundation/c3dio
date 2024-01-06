//! Includes the analog data and parameters.
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::data::{get_analog_bytes_per_frame, get_point_bytes_per_frame, DataFormat};
use crate::parameters::{Parameter, ParameterData, Parameters};
use crate::processor::Processor;
use crate::{C3dParseError, C3dWriteError};
use grid::Grid;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum AnalogFormat {
    #[default]
    Signed,
    Unsigned,
}

impl AnalogFormat {
    pub(crate) fn from_parameters(
        parameters: &mut Parameters,
    ) -> Result<AnalogFormat, C3dParseError> {
        let analog_format_parameter_data = parameters.remove("ANALOG", "FORMAT");
        match analog_format_parameter_data {
            Some(analog_format_parameter_data) => {
                let analog_format_parameter_data: String =
                    analog_format_parameter_data.as_ref().try_into()?;
                match analog_format_parameter_data.as_str() {
                    "SIGNED" => Ok(AnalogFormat::Signed),
                    "UNSIGNED" => Ok(AnalogFormat::Unsigned),
                    _ => Ok(AnalogFormat::Signed),
                }
            }
            None => Ok(AnalogFormat::Signed),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnalogOffset {
    Signed(Vec<i16>),
    Unsigned(Vec<u16>),
}

impl Default for AnalogOffset {
    fn default() -> Self {
        AnalogOffset::Signed(Vec::new())
    }
}

impl AnalogOffset {
    pub(crate) fn from_parameters(
        parameters: &mut Parameters,
        format: &AnalogFormat,
    ) -> Result<AnalogOffset, C3dParseError> {
        let offset = parameters.remove("ANALOG", "OFFSET");
        let offset = match offset {
            Some(offset) => offset,
            None => return Ok(AnalogOffset::Signed(Vec::new())),
        };
        match format {
            AnalogFormat::Signed => Ok(AnalogOffset::Signed(offset.as_ref().try_into()?)),
            AnalogFormat::Unsigned => {
                let offset: Vec<i16> = offset.as_ref().try_into()?;
                let offset: Vec<u16> = offset.iter().map(|x| *x as u16).collect();
                Ok(AnalogOffset::Unsigned(offset))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Analog {
    pub parsed_header: bool,
    pub analog: Grid<f64>,
    pub labels: Vec<String>,
    pub descriptions: Vec<String>,
    pub units: Vec<String>,
    pub scales: Vec<f32>,
    pub rate: f32,
    pub samples_per_channel_per_frame: u16,
    pub samples_per_frame: u16,
    pub offset: AnalogOffset,
    pub gen_scale: f32,
    pub bits: i16,
}

impl PartialEq for Analog {
    fn eq(&self, other: &Self) -> bool {
        self.parsed_header == other.parsed_header
            && self.analog.flatten() == other.analog.flatten()
            && self.labels == other.labels
            && self.descriptions == other.descriptions
            && self.units == other.units
            && self.scales == other.scales
            && self.rate == other.rate
            && self.samples_per_channel_per_frame == other.samples_per_channel_per_frame
            && self.samples_per_frame == other.samples_per_frame
            && self.offset == other.offset
            && self.gen_scale == other.gen_scale
            && self.bits == other.bits
    }
}

impl Default for Analog {
    fn default() -> Self {
        Analog {
            parsed_header: false,
            analog: Grid::new(0, 0),
            labels: Vec::new(),
            descriptions: Vec::new(),
            units: Vec::new(),
            scales: Vec::new(),
            rate: 0.0,
            samples_per_channel_per_frame: 0,
            samples_per_frame: 0,
            offset: AnalogOffset::Signed(Vec::new()),
            gen_scale: 0.0,
            bits: 0,
        }
    }
}

impl Deref for Analog {
    type Target = Grid<f64>;

    fn deref(&self) -> &Self::Target {
        &self.analog
    }
}

impl DerefMut for Analog {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.analog
    }
}

impl ToString for Analog {
    fn to_string(&self) -> String {
        let mut string = String::new();
        string.push_str("Analog Data:\n");
        string.push_str(&format!("Size: {:?}\n", self.analog.size()));
        string.push_str(&format!("Labels: {:?}\n", self.labels));
        string.push_str(&format!("Descriptions: {:?}\n", self.descriptions));
        string.push_str(&format!("Units: {:?}\n", self.units));
        string.push_str(&format!("Scales: {:?}\n", self.scales));
        string.push_str(&format!("Rate: {}\n", self.rate));
        string.push_str(&format!(
            "Samples per channel per frame: {}\n",
            self.samples_per_channel_per_frame
        ));
        string.push_str(&format!("Samples per frame: {}\n", self.samples_per_frame));
        string.push_str(&format!("Offset: {:?}\n", self.offset));
        string.push_str(&format!("Gen scale: {}\n", self.gen_scale));
        string.push_str(&format!("Bits: {}\n", self.bits));
        string
    }
}

impl Analog {
    pub(crate) fn new() -> Self {
        Analog::default()
    }

    pub(crate) fn parse_header(header: &[u8; 512], processor: &Processor) -> Analog {
        let mut analog = Analog::new();
        analog.samples_per_frame = processor.u16([header[4], header[5]]);
        analog.samples_per_channel_per_frame = processor.u16([header[18], header[19]]);
        analog
    }

    pub(crate) fn parse(
        &mut self,
        data_bytes: &Vec<u8>,
        parameters: &mut Parameters,
        processor: &Processor,
        num_frames: usize,
        format: &DataFormat,
        points_per_frame: usize,
    ) -> Result<&mut Self, C3dParseError> {
        let analog_used = self.get_analog_parameters(parameters)?;
        self.parse_analog(
            data_bytes,
            processor,
            num_frames,
            format,
            points_per_frame,
            analog_used,
        )
    }

    pub(crate) fn write_parameters(
        &self,
        processor: &Processor,
        group_names_to_ids: &HashMap<String, usize>,
    ) -> Result<Vec<u8>, C3dWriteError> {
        let mut bytes = Vec::new();
        // "ANALOG", "USED"
        bytes.extend(Parameter::integer(self.analog.cols() as i16).write(
            processor,
            "USED".to_string(),
            group_names_to_ids["ANALOG"],
            false,
        )?);
        // "ANALOG", "LABELS"
        bytes.extend(Parameter::strings(self.labels.clone()).write(
            processor,
            "LABELS".to_string(),
            group_names_to_ids["ANALOG"],
            false,
        )?);
        // "ANALOG", "DESCRIPTIONS"
        bytes.extend(Parameter::strings(self.descriptions.clone()).write(
            processor,
            "DESCRIPTIONS".to_string(),
            group_names_to_ids["ANALOG"],
            false,
        )?);
        // "ANALOG", "GEN_SCALE"
        bytes.extend(Parameter::float(self.gen_scale).write(
            processor,
            "GEN_SCALE".to_string(),
            group_names_to_ids["ANALOG"],
            false,
        )?);
        // "ANALOG", "UNITS"
        bytes.extend(Parameter::strings(self.units.clone()).write(
            processor,
            "UNITS".to_string(),
            group_names_to_ids["ANALOG"],
            false,
        )?);
        // "ANALOG", "SCALE"
        if self.scales.len() != 0 {
            bytes.extend(Parameter::floats(self.scales.clone())?.write(
                processor,
                "SCALE".to_string(),
                group_names_to_ids["ANALOG"],
                false,
            )?);
        }
        // "ANALOG", "RATE"
        bytes.extend(Parameter::float(self.rate).write(
            processor,
            "RATE".to_string(),
            group_names_to_ids["ANALOG"],
            false,
        )?);
        // "ANALOG", "FORMAT"
        match self.offset {
            AnalogOffset::Signed(_) => {
                bytes.extend(Parameter::string("SIGNED".to_string())?.write(
                    processor,
                    "FORMAT".to_string(),
                    group_names_to_ids["ANALOG"],
                    false,
                )?);
            }
            AnalogOffset::Unsigned(_) => {
                bytes.extend(Parameter::string("UNSIGNED".to_string())?.write(
                    processor,
                    "FORMAT".to_string(),
                    group_names_to_ids["ANALOG"],
                    false,
                )?);
            }
        }
        // "ANALOG", "BITS"
        bytes.extend(Parameter::integer(self.bits).write(
            processor,
            "BITS".to_string(),
            group_names_to_ids["ANALOG"],
            false,
        )?);
        // "ANALOG", "OFFSET"
        let offset: Vec<i16> = match &self.offset {
            AnalogOffset::Signed(offset) => offset.iter().map(|x| *x as i16).collect(),
            AnalogOffset::Unsigned(offset) => offset.iter().map(|x| *x as i16).collect(),
        };
        if offset.len() != 0 {
            bytes.extend(Parameter::integers(offset)?.write(
                processor,
                "OFFSET".to_string(),
                group_names_to_ids["ANALOG"],
                false,
            )?);
        }
        Ok(bytes)
    }

    pub(crate) fn write_frame(
        &self,
        frame: usize,
        processor: &Processor,
        data_format: &DataFormat,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();

        let start_row = frame * self.samples_per_channel_per_frame as usize;
        let end_row = start_row + self.samples_per_channel_per_frame as usize;
        if end_row > self.analog.size().0 {
            return bytes;
        }
        for i in start_row..end_row {
            for (column, value) in self.analog.iter_row(i).enumerate() {
                match data_format {
                    DataFormat::Float => {
                        let offset = match &self.offset {
                            AnalogOffset::Signed(offset) => offset[column] as f64,
                            AnalogOffset::Unsigned(offset) => offset[column] as f64,
                        };
                        let value = value / self.scales[column] as f64 / self.gen_scale as f64
                            + offset as f64;
                        let value = processor.f32_to_bytes(value as f32);
                        bytes.extend_from_slice(&value);
                    }
                    DataFormat::Integer => match &self.offset {
                        AnalogOffset::Signed(offset) => {
                            let offset = offset[column] as f64;
                            let value = value / self.scales[column] as f64 / self.gen_scale as f64
                                + offset as f64;
                            let value = value.round() as i16;
                            let value = processor.i16_to_bytes(value);
                            bytes.extend_from_slice(&value);
                        }
                        AnalogOffset::Unsigned(offset) => {
                            let offset = offset[column] as f64;
                            let value = value / self.scales[column] as f64 / self.gen_scale as f64
                                + offset as f64;
                            let value = value.round() as u16;
                            let value = processor.u16_to_bytes(value);
                            bytes.extend_from_slice(&value);
                        }
                    },
                }
            }
        }
        bytes
    }

    fn get_analog_parameters(&mut self, parameters: &mut Parameters) -> Result<u16, C3dParseError> {
        let analog_format = AnalogFormat::from_parameters(parameters)?;
        self.offset = AnalogOffset::from_parameters(parameters, &analog_format)?;
        let used = parameters.remove("ANALOG", "USED");
        let mut is_none_or_zero = used.is_none();
        if !is_none_or_zero {
            let used = used.clone().unwrap();
            let used: u16 = used.as_ref().try_into()?;
            is_none_or_zero = used == 0;
        }
        if is_none_or_zero {
            self.labels = Vec::new();
            self.descriptions = Vec::new();
            self.gen_scale = 0.0;
            self.units = Vec::new();
            self.scales = Vec::new();
            self.rate = 0.0;
            self.bits = 0;
            return Ok(0);
        } else {
            self.labels = parameters
                .remove_or_err("ANALOG", "LABELS")?
                .as_ref()
                .try_into()?;
            self.descriptions = parameters
                .remove("ANALOG", "DESCRIPTIONS")
                .unwrap_or(Parameter::strings(vec![" ".to_string()]))
                .as_ref()
                .try_into()?;
            self.gen_scale = parameters
                .remove_or_err("ANALOG", "GEN_SCALE")?
                .as_ref()
                .try_into()?;
            self.units = parameters
                .remove_or_err("ANALOG", "UNITS")?
                .as_ref()
                .try_into()?;
            self.scales = parameters
                .remove_or_err("ANALOG", "SCALE")?
                .as_ref()
                .try_into()?;
            self.rate = parameters
                .remove_or_err("ANALOG", "RATE")?
                .as_ref()
                .try_into()?;
            let bits = parameters.remove("ANALOG", "BITS");
            if bits.is_none() {
                self.bits = 12;
            } else {
                self.bits = match &bits.unwrap().data {
                    ParameterData::Integer(bits) => bits[0],
                    ParameterData::Byte(bits) => bits[0] as i16,
                    ParameterData::Float(bits) => bits[0] as i16,
                    _ => {
                        return Err(C3dParseError::InvalidParameterType(
                            "ANALOG:BITS".to_string(),
                        ))
                    }
                };
            }
        }
        Ok(used.unwrap().as_ref().try_into()?)
    }

    fn parse_analog(
        &mut self,
        data_bytes: &Vec<u8>,
        processor: &Processor,
        num_frames: usize,
        format: &DataFormat,
        points_per_frame: usize,
        analog_used: u16,
    ) -> Result<&mut Self, C3dParseError> {
        let mut analog_data = Grid::new(
            num_frames * self.samples_per_channel_per_frame as usize,
            analog_used as usize,
        );

        let point_bytes_per_frame = get_point_bytes_per_frame(format, points_per_frame) as usize;

        let analog_bytes_per_frame = get_analog_bytes_per_frame(format, self.samples_per_frame)?;
        let bytes_per_frame = point_bytes_per_frame + analog_bytes_per_frame;
        let bytes_per_analog_point = match self.samples_per_frame {
            0 => 0,
            _ => analog_bytes_per_frame / self.samples_per_frame as usize,
        };
        if analog_bytes_per_frame
            != bytes_per_analog_point
                * analog_used as usize
                * self.samples_per_channel_per_frame as usize
        {
            return Err(C3dParseError::AnalogBytesPerFrameMismatch);
        }
        for i in 0..num_frames {
            let start = i * bytes_per_frame as usize;
            let end = start + bytes_per_frame as usize;
            let analog_frame_data = &data_bytes[start + point_bytes_per_frame as usize..end];
            for j in 0..self.samples_per_channel_per_frame {
                let start = j as usize * bytes_per_analog_point * analog_used as usize;
                let end = start + (bytes_per_analog_point * analog_used as usize);
                let analog_slice = &analog_frame_data[start as usize..end as usize];
                let temp_analog_data = match format {
                    DataFormat::Float => {
                        parse_analog_data_float(analog_slice, analog_used as usize, processor)
                    }
                    DataFormat::Integer => {
                        parse_analog_data_int(analog_slice, analog_used as usize, processor)
                    }
                };
                for k in 0..analog_used as usize {
                    let row = i * self.samples_per_channel_per_frame as usize + j as usize;
                    analog_data[row][k] = temp_analog_data[k] as f64;
                }
            }
        }
        let offset_len = match &self.offset {
            AnalogOffset::Signed(offset) => offset.len(),
            AnalogOffset::Unsigned(offset) => offset.len(),
        };
        if analog_used > 0 && offset_len != self.scales.len() {
            return Err(C3dParseError::AnalogOffsetScaleMismatch);
        }
        if analog_used as usize <= offset_len {
            for i in 0..analog_data.cols() {
                let col_iter = analog_data.iter_col_mut(i);
                match &self.offset {
                    AnalogOffset::Signed(offset) => col_iter.for_each(|x| {
                        *x -= offset[i] as f64;
                        *x *= self.scales[i] as f64 * self.gen_scale as f64;
                    }),
                    AnalogOffset::Unsigned(offset) => col_iter.for_each(|x| {
                        *x -= offset[i] as f64;
                        *x *= self.scales[i] as f64 * self.gen_scale as f64;
                    }),
                };
            }
        } else {
            return Err(C3dParseError::InsufficientAnalogOffsets);
        }
        self.analog = analog_data;
        Ok(self)
    }
}

fn parse_analog_data_float(
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

fn parse_analog_data_int(
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
