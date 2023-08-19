use std::collections::HashMap;

use nalgebra::{DMatrix, DVector, Matrix6};

use crate::processor::Processor;
use crate::C3dParseError;
//use ndarray::{Array, Array2, Array3, ArrayView, Ix2, Ix3, IxDyn, Order};

#[derive(Debug, Clone)]
pub struct Parameters {
    pub group_descriptions: HashMap<String, String>,
    pub raw_parameters: HashMap<String, HashMap<String, (ParameterData, String)>>,
    pub point: PointParameters,
    pub analog: AnalogParameters,
    pub force_platform: ForcePlatformParameters,
    pub trial: TrialParameters,
    pub event: EventParameters,
    pub event_context: EventContextParameters,
    pub manufacturer: ManufacturerParameters,
    pub seg: SegParameters,
}

impl Parameters {
    pub fn new() -> Parameters {
        Parameters {
            group_descriptions: HashMap::new(),
            raw_parameters: HashMap::new(),
            point: PointParameters::default(),
            analog: AnalogParameters::default(),
            force_platform: ForcePlatformParameters::default(),
            trial: TrialParameters::new(),
            event: EventParameters::new(),
            event_context: EventContextParameters::new(),
            manufacturer: ManufacturerParameters::new(),
            seg: SegParameters::default(),
        }
    }

    pub fn parse_parameter_blocks(
        parameter_blocks: &Vec<u8>,
        processor: &Processor,
    ) -> Result<Self, C3dParseError> {
        let (raw_parameters, group_descriptions) =
            parse_parameter_blocks(parameter_blocks, processor)?;
        let point = PointParameters::from_raw(&raw_parameters)?;
        let analog = AnalogParameters::from_raw(&raw_parameters)?;
        let force_platform = ForcePlatformParameters::from_raw(&raw_parameters)?;
        let trial = TrialParameters::from_raw(&raw_parameters)?;
        let event = EventParameters::from_raw(&raw_parameters);
        let event_context = EventContextParameters::from_raw(&raw_parameters);
        let manufacturer = ManufacturerParameters::from_raw(&raw_parameters);
        let seg = SegParameters::from_raw(&raw_parameters);
        Ok(Parameters {
            group_descriptions,
            raw_parameters,
            point,
            analog,
            force_platform,
            trial,
            event,
            event_context,
            manufacturer,
            seg,
        })
    }

    pub fn get_int_vec(&self, group: &str, parameter: &str) -> Option<Vec<i16>> {
        get_signed_integer_vec(&self.raw_parameters, group, parameter)
    }

    pub fn get_data(&self, group: &str, parameter: &str) -> Option<ParameterData> {
        get(&self.raw_parameters, group, parameter)
    }
}

impl PartialEq for Parameters {
    fn eq(&self, other: &Self) -> bool {
        self.group_descriptions == other.group_descriptions
            && self.point == other.point
            && self.analog == other.analog
            && self.force_platform == other.force_platform
            && self.trial == other.trial
            && self.event == other.event
            && self.event_context == other.event_context
            && self.manufacturer == other.manufacturer
            && self.seg == other.seg
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
    fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
        format: &AnalogFormat,
    ) -> Result<AnalogOffset, C3dParseError> {
        let offset = get_signed_integer_vec(raw_parameters, "ANALOG", "OFFSET");
        let offset = match offset {
            Some(offset) => offset,
            None => return Ok(AnalogOffset::Signed(Vec::new())),
        };
        match format {
            AnalogFormat::Signed => Ok(AnalogOffset::Signed(offset)),
            AnalogFormat::Unsigned => {
                let offset: Vec<u16> = offset.iter().map(|x| *x as u16).collect();
                Ok(AnalogOffset::Unsigned(offset))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForcePlatformType {
    Type1,
    Type2,
    Type3,
    Type4,
}

impl ForcePlatformType {
    fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    ) -> Result<Vec<ForcePlatformType>, C3dParseError> {
        let force_platform_type: Vec<i16> =
            get_or_err(raw_parameters, "FORCE_PLATFORM", "TYPE")?.try_into()?;
        let force_platform_type = force_platform_type
            .iter()
            .map(|x| match x {
                1 => Ok(ForcePlatformType::Type1),
                2 => Ok(ForcePlatformType::Type2),
                3 => Ok(ForcePlatformType::Type3),
                4 => Ok(ForcePlatformType::Type4),
                _ => Err(C3dParseError::InvalidParameterFormat(x.to_string())),
            })
            .collect::<Result<Vec<ForcePlatformType>, C3dParseError>>()?;
        Ok(force_platform_type)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PointParameters {
    pub used: u16,
    pub scale: f32,
    pub rate: f32,
    pub data_start: u16,
    pub frames: u16,
    pub labels: Vec<String>,
    pub descriptions: Vec<String>,
    pub units: [char; 4],
    pub long_frames: Option<f32>,
    pub x_screen: Option<[char; 2]>,
    pub y_screen: Option<[char; 2]>,
}

impl PartialEq for PointParameters {
    fn eq(&self, other: &Self) -> bool {
        self.used == other.used
//            && self.scale == other.scale
            && self.rate == other.rate
//            && self.data_start == other.data_start
            && self.frames == other.frames
            && self.labels == other.labels
            && self.descriptions == other.descriptions
            && self.units == other.units
            && self.long_frames == other.long_frames
            && self.x_screen == other.x_screen
            && self.y_screen == other.y_screen
    }
}

impl PointParameters {
    pub fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    ) -> Result<Self, C3dParseError> {
        let used = get_signed_integer(raw_parameters, "POINT", "USED");
        if used.is_none() || used.unwrap() == 0 {
            return Ok(PointParameters::default());
        }
        let mut frames = get_signed_integer(raw_parameters, "POINT", "FRAMES");
        if frames.is_none() {
            let try_float = get_float(raw_parameters, "POINT", "FRAMES").map(|x| x as i16);
            if try_float.is_none() {
                return Err(C3dParseError::ParameterNotFound(
                    "POINT".to_string(),
                    "FRAMES".to_string(),
                ));
            }
            frames = try_float;
        }
        let required_point_parameters = PointParameters {
            used: get_or_err(raw_parameters, "POINT", "USED")?.try_into()?,
            scale: get_or_err(raw_parameters, "POINT", "SCALE")?.try_into()?,
            rate: get_float(raw_parameters, "POINT", "RATE").unwrap_or(0.0),
            data_start: get_or_err(raw_parameters, "POINT", "DATA_START")?.try_into()?,
            frames: frames.unwrap() as u16,
            labels: get_string_vec(raw_parameters, "POINT", "LABELS").unwrap_or(Vec::new()),
            descriptions: get_string_vec(raw_parameters, "POINT", "DESCRIPTIONS")
                .unwrap_or(Vec::new()),
            units: get_char_quad(raw_parameters, "POINT", "UNITS").unwrap_or([' '; 4]),
            long_frames: get_float(raw_parameters, "POINT", "LONG_FRAMES"),
            x_screen: get_char_pair(raw_parameters, "POINT", "X_SCREEN"),
            y_screen: get_char_pair(raw_parameters, "POINT", "Y_SCREEN"),
        };
        Ok(required_point_parameters)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum AnalogFormat {
    #[default]
    Signed,
    Unsigned,
}

impl AnalogFormat {
    fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    ) -> AnalogFormat {
        let analog_format_parameter_data = get_string(raw_parameters, "ANALOG", "FORMAT");
        match analog_format_parameter_data {
            Some(analog_format_parameter_data) => match analog_format_parameter_data.as_str() {
                "SIGNED" => AnalogFormat::Signed,
                "UNSIGNED" => AnalogFormat::Unsigned,
                _ => AnalogFormat::Signed,
            },
            None => AnalogFormat::Signed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AnalogParameters {
    pub used: u16,
    pub labels: Vec<String>,
    pub descriptions: Vec<String>,
    pub gen_scale: f32,
    pub offset: AnalogOffset,
    pub units: Vec<String>,
    pub scale: Vec<f32>,
    pub rate: f32,
    pub format: AnalogFormat,
    pub bits: i16,
    pub gain: Option<Vec<i16>>,
}

impl AnalogParameters {
    pub fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    ) -> Result<Self, C3dParseError> {
        let used = get_signed_integer(raw_parameters, "ANALOG", "USED");
        if used.is_none() || used.unwrap() == 0 {
            return Ok(AnalogParameters::default());
        }
        let format = AnalogFormat::from_raw(raw_parameters);
        let offset = AnalogOffset::from_raw(raw_parameters, &format)?;
        let analog_parameters = AnalogParameters {
            used: get_or_err(raw_parameters, "ANALOG", "USED")?.try_into()?,
            labels: get_or_err(raw_parameters, "ANALOG", "LABELS")?.try_into()?,
            descriptions: get_string_vec(raw_parameters, "ANALOG", "DESCRIPTIONS")
                .unwrap_or(Vec::new()),
            gen_scale: get_or_err(raw_parameters, "ANALOG", "GEN_SCALE")?.try_into()?,
            offset,
            units: get_or_err(raw_parameters, "ANALOG", "UNITS")?.try_into()?,
            scale: get_or_err(raw_parameters, "ANALOG", "SCALE")?.try_into()?,
            rate: get_or_err(raw_parameters, "ANALOG", "RATE")?.try_into()?,
            format,
            bits: get_signed_integer(raw_parameters, "ANALOG", "BITS").unwrap_or(12),
            gain: get_signed_integer_vec(raw_parameters, "ANALOG", "GAIN"),
        };
        Ok(analog_parameters)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForcePlatformCorners {
    corners: Vec<[f32; 12]>,
}

impl ForcePlatformCorners {
    pub fn new() -> Self {
        ForcePlatformCorners {
            corners: Vec::new(),
        }
    }

    pub fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    ) -> Result<Self, C3dParseError> {
        let mut corners = Vec::new();
        let data = get_or_err(raw_parameters, "FORCE_PLATFORM", "CORNERS")?;
        match data {
            ParameterData::Float(data, dimensions) => {
                if dimensions.len() == 3 {
                    for i in 0..dimensions[2] {
                        let mut corner = [0.0; 12];
                        for j in 0..dimensions[0] {
                            for k in 0..dimensions[1] {
                                let index =
                                    i * (dimensions[0] * dimensions[1]) + j * dimensions[1] + k;
                                corner[j * dimensions[1] + k] = data[index];
                            }
                        }
                        corners.push(corner);
                    }
                } else {
                    return Err(C3dParseError::InvalidData(
                        "FORCE_PLATFORM_CORNERS".to_string(),
                    ));
                }
            }
            _ => {
                return Err(C3dParseError::InvalidData(
                    "FORCE_PLATFORM_CORNERS".to_string(),
                ));
            }
        }
        Ok(ForcePlatformCorners { corners })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForcePlatformOrigin {
    origin: Vec<[f32; 3]>,
}

impl ForcePlatformOrigin {
    pub fn new() -> Self {
        ForcePlatformOrigin { origin: Vec::new() }
    }

    pub fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    ) -> Result<Self, C3dParseError> {
        let mut origin = Vec::new();
        let data = get_or_err(raw_parameters, "FORCE_PLATFORM", "ORIGIN")?;
        match data {
            ParameterData::Float(data, dimensions) => {
                if dimensions.len() == 2 {
                    for i in 0..dimensions[1] {
                        let mut origin_point = [0.0; 3];
                        for j in 0..dimensions[0] {
                            origin_point[j] = data[j * dimensions[1] + i];
                        }
                        origin.push(origin_point);
                    }
                } else {
                    return Err(C3dParseError::InvalidData(
                        "FORCE_PLATFORM_ORIGIN".to_string(),
                    ));
                }
            }
            _ => {
                return Err(C3dParseError::InvalidData(
                    "FORCE_PLATFORM_ORIGIN".to_string(),
                ));
            }
        }
        Ok(ForcePlatformOrigin { origin })
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ForcePlatformParameters {
    pub used: u16,
    pub plate_type: Option<Vec<ForcePlatformType>>,
    pub zero: Option<[u16; 2]>,
    pub corners: Option<ForcePlatformCorners>,
    pub origin: Option<ForcePlatformOrigin>,
    pub channel: Option<DMatrix<i16>>,
    pub cal_matrix: Option<Vec<Matrix6<f32>>>,
}

impl ForcePlatformParameters {
    pub fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    ) -> Result<Self, C3dParseError> {
        let used = get_signed_integer(raw_parameters, "FORCE_PLATFORM", "USED");
        if used.is_none() || used.unwrap() == 0 {
            return Ok(ForcePlatformParameters::default());
        }
        let plate_type = ForcePlatformType::from_raw(raw_parameters)?;
        let corners = ForcePlatformCorners::from_raw(raw_parameters)?;
        let origin = ForcePlatformOrigin::from_raw(raw_parameters)?;
        let zero = get_or_err(raw_parameters, "FORCE_PLATFORM", "ZERO")?.try_into()?;
        let channel = get_signed_integer_array2(raw_parameters, "FORCE_PLATFORM", "CHANNEL");
        let force_platform_parameters = ForcePlatformParameters {
            used: used.unwrap() as u16,
            plate_type: Some(plate_type),
            zero: Some(zero),
            corners: Some(corners),
            origin: Some(origin),
            channel,
            cal_matrix: get_cal_matrix_vector(raw_parameters, "FORCE_PLATFORM", "CAL_MATRIX"),
        };
        Ok(force_platform_parameters)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrialParameters {
    pub actual_start_field: Option<usize>,
    pub actual_end_field: Option<usize>,
    pub camera_rate: Option<f32>,
}

impl TrialParameters {
    pub fn new() -> Self {
        TrialParameters {
            actual_start_field: None,
            actual_end_field: None,
            camera_rate: None,
        }
    }

    pub fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    ) -> Result<Self, C3dParseError> {
        let end_field = get_signed_integer_vec(raw_parameters, "TRIAL", "ACTUAL_END_FIELD");
        let actual_end_field = if end_field.is_some() {
            let end_field = end_field.unwrap();
            if end_field.len() != 2 {
                None
            } else {
                Some(end_field[0] as u16 as usize + (end_field[1] as u16 * 65535) as usize)
            }
        } else {
            None
        };
        let start_field = get_signed_integer_vec(raw_parameters, "TRIAL", "ACTUAL_START_FIELD");
        let actual_start_field = if start_field.is_some() {
            let start_field = start_field.unwrap();
            if start_field.len() != 2 {
                None
            } else {
                Some(start_field[0] as u16 as usize + (start_field[1] as u16 * 65535) as usize)
            }
        } else {
            None
        };
        let trial_parameters = TrialParameters {
            actual_end_field,
            actual_start_field,
            camera_rate: get_float(raw_parameters, "TRIAL", "CAMERA_RATE"),
        };
        Ok(trial_parameters)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventParameters {
    pub used: Option<i16>,
    pub contexts: Option<Vec<String>>,
    pub labels: Option<Vec<String>>,
    pub descriptions: Option<Vec<String>>,
    pub times: Option<Vec<[f32; 2]>>,
    pub subjects: Option<Vec<String>>,
    pub icon_ids: Option<Vec<i16>>,
    pub generic_flags: Option<Vec<i16>>,
}

impl EventParameters {
    pub fn new() -> Self {
        EventParameters {
            used: None,
            contexts: None,
            labels: None,
            descriptions: None,
            times: None,
            subjects: None,
            icon_ids: None,
            generic_flags: None,
        }
    }

    pub fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    ) -> Self {
        EventParameters {
            used: get_signed_integer(raw_parameters, "EVENT", "USED"),
            contexts: get_string_vec(raw_parameters, "EVENT", "CONTEXTS"),
            labels: get_string_vec(raw_parameters, "EVENT", "LABELS"),
            descriptions: get_string_vec(raw_parameters, "EVENT", "DESCRIPTIONS"),
            times: get_times_array(raw_parameters, "EVENT", "TIMES"),
            subjects: get_string_vec(raw_parameters, "EVENT", "SUBJECTS"),
            icon_ids: get_signed_integer_vec(raw_parameters, "EVENT", "ICON_IDS"),
            generic_flags: get_signed_integer_vec(raw_parameters, "EVENT", "GENERIC_FLAGS"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventContextParameters {
    pub used: Option<i16>,
    pub icon_ids: Option<Vec<u16>>,
    pub labels: Option<Vec<String>>,
    pub descriptions: Option<Vec<String>>,
    pub colours: Option<Vec<[u8; 3]>>,
}

impl EventContextParameters {
    pub fn new() -> Self {
        EventContextParameters {
            used: None,
            icon_ids: None,
            labels: None,
            descriptions: None,
            colours: None,
        }
    }

    pub fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    ) -> Self {
        EventContextParameters {
            used: get_signed_integer(raw_parameters, "EVENT_CONTEXT", "USED"),
            icon_ids: get_signed_integer_vec(raw_parameters, "EVENT_CONTEXT", "ICON_IDS")
                .map(|x| x.iter().map(|&x| x as u16).collect()),
            labels: get_string_vec(raw_parameters, "EVENT_CONTEXT", "LABELS"),
            descriptions: get_string_vec(raw_parameters, "EVENT_CONTEXT", "DESCRIPTIONS"),
            colours: get_colour_array(raw_parameters, "EVENT_CONTEXT", "COLOURS"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ManufacturerVersion {
    String(String),
    Float(f32),
    Array(Vec<u16>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManufacturerParameters {
    pub company: Option<String>,
    pub software: Option<String>,
    pub version: Option<ManufacturerVersion>,
    pub edited: Option<Vec<String>>,
}

impl ManufacturerParameters {
    pub fn new() -> Self {
        ManufacturerParameters {
            company: None,
            software: None,
            version: None,
            edited: None,
        }
    }

    pub fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    ) -> Self {
        ManufacturerParameters {
            company: get_string(raw_parameters, "MANUFACTURER", "COMPANY"),
            software: get_string(raw_parameters, "MANUFACTURER", "SOFTWARE"),
            version: None, // TODO
            edited: get_string_vec(raw_parameters, "MANUFACTURER", "EDITED"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SegParameters {
    pub marker_diameter: Option<f32>,
    pub data_limits: Option<DMatrix<f32>>,
    pub acc_factor: Option<f32>,
    pub noise_factor: Option<f32>,
    pub residual_error_factor: Option<f32>,
    pub intersection_limit: Option<f32>,
}

impl SegParameters {
    pub fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    ) -> Self {
        SegParameters {
            marker_diameter: get_float(raw_parameters, "SEG", "MARKER_DIAMETER"),
            data_limits: get_float_array2(raw_parameters, "SEG", "DATA_LIMITS"),
            acc_factor: get_float(raw_parameters, "SEG", "ACC_FACTOR"),
            noise_factor: get_float(raw_parameters, "SEG", "NOISE_FACTOR"),
            residual_error_factor: get_float(raw_parameters, "SEG", "RESIDUAL_ERROR_FACTOR"),
            intersection_limit: get_float(raw_parameters, "SEG", "INTERSECTION_LIMIT"),
        }
    }
}

impl TryFrom<ParameterData> for u16 {
    type Error = C3dParseError;
    fn try_from(data: ParameterData) -> Result<Self, Self::Error> {
        match data {
            ParameterData::Integer(data, dimensions) => {
                if data.len() == 1 {
                    Ok(data[0] as u16)
                } else {
                    Err(C3dParseError::InvalidData("u16".to_string()))
                }
            }
            _ => Err(C3dParseError::InvalidData("u16".to_string())),
        }
    }
}

impl TryFrom<ParameterData> for [u16; 2] {
    type Error = C3dParseError;
    fn try_from(data: ParameterData) -> Result<Self, Self::Error> {
        match data {
            ParameterData::Integer(data, dimensions) => {
                if data.len() == 2 {
                    Ok([data[0] as u16, data[1] as u16])
                } else {
                    Err(C3dParseError::InvalidData("u162".to_string()))
                }
            }
            _ => Err(C3dParseError::InvalidData("u162".to_string())),
        }
    }
}

impl TryFrom<ParameterData> for Vec<u16> {
    type Error = C3dParseError;
    fn try_from(data: ParameterData) -> Result<Self, Self::Error> {
        match data {
            ParameterData::Integer(data, dimensions) => {
                if dimensions.len() == 1 {
                    Ok(data.iter().map(|x| *x as u16).collect())
                } else {
                    Err(C3dParseError::InvalidData("Vec<u16>".to_string()))
                }
            }
            _ => Err(C3dParseError::InvalidData("Vec<u16>".to_string())),
        }
    }
}

impl TryFrom<ParameterData> for i16 {
    type Error = C3dParseError;
    fn try_from(data: ParameterData) -> Result<Self, Self::Error> {
        match data {
            ParameterData::Integer(data, dimensions) => {
                if data.len() == 1 {
                    Ok(data[0])
                } else {
                    Err(C3dParseError::InvalidData("i16".to_string()))
                }
            }
            _ => Err(C3dParseError::InvalidData("i16".to_string())),
        }
    }
}

impl TryFrom<ParameterData> for Vec<i16> {
    type Error = C3dParseError;
    fn try_from(data: ParameterData) -> Result<Self, Self::Error> {
        match data {
            ParameterData::Integer(data, dimensions) => {
                if dimensions.len() == 1 {
                    Ok(data.into_iter().map(|&x| x as i16).collect())
                } else {
                    Err(C3dParseError::InvalidData("Vec<i16>".to_string()))
                }
            }
            _ => Err(C3dParseError::InvalidData("Vec<i16>".to_string())),
        }
    }
}

impl TryFrom<ParameterData> for f32 {
    type Error = C3dParseError;
    fn try_from(data: ParameterData) -> Result<Self, Self::Error> {
        match data {
            ParameterData::Float(data, dimensions) => {
                if data.len() == 1 {
                    Ok(data[0])
                } else {
                    Err(C3dParseError::InvalidData("f32".to_string()))
                }
            }
            _ => Err(C3dParseError::InvalidData("f32".to_string())),
        }
    }
}

impl TryFrom<ParameterData> for Vec<f32> {
    type Error = C3dParseError;
    fn try_from(data: ParameterData) -> Result<Self, Self::Error> {
        match data {
            ParameterData::Float(data, dimensions) => {
                if dimensions.len() == 1 {
                    Ok(data.into_iter().map(|&x| x as f32).collect())
                } else {
                    Err(C3dParseError::InvalidData("Vec<f32>".to_string()))
                }
            }
            _ => Err(C3dParseError::InvalidData("Vec<f32>".to_string())),
        }
    }
}

impl TryFrom<ParameterData> for String {
    type Error = C3dParseError;
    fn try_from(data: ParameterData) -> Result<Self, Self::Error> {
        match data {
            ParameterData::Char(data, dimensions) => {
                let mut string = String::new();
                for c in 0..data.len() {
                    string.push(data[c]);
                }
                Ok(string)
            }
            _ => Err(C3dParseError::InvalidData("String".to_string())),
        }
    }
}

impl TryFrom<ParameterData> for Vec<String> {
    type Error = C3dParseError;
    fn try_from(data: ParameterData) -> Result<Self, Self::Error> {
        match data {
            ParameterData::Char(data, dimensions) => {
                if dimensions.len() == 2 || dimensions.len() == 1 {
                    let mut strings = Vec::new();
                    let num_strings = match dimensions.len() == 1 {
                        True => 1,
                        False => dimensions[1],
                    };
                    for row in 0..num_strings {
                        let mut string = String::new();
                        for c in 0..dimensions[0] {
                            string.push(data[row * dimensions[0] + c]);
                        }
                        strings.push(string);
                    }
                    Ok(strings)
                } else {
                    Err(C3dParseError::InvalidData("Vec<String>".to_string()))
                }
            }
            _ => Err(C3dParseError::InvalidData("Vec<String>".to_string())),
        }
    }
}

impl TryFrom<ParameterData> for [char; 4] {
    type Error = C3dParseError;
    fn try_from(data: ParameterData) -> Result<Self, Self::Error> {
        match data {
            ParameterData::Char(data, dimensions) => {
                let mut chars = [' '; 4];
                for (i, c) in data.iter().enumerate() {
                    if i >= 4 {
                        break;
                    }
                    chars[i] = *c;
                }
                Ok(chars)
            }
            _ => Err(C3dParseError::InvalidData("char4".to_string())),
        }
    }
}

impl TryFrom<ParameterData> for Vec<[char; 4]> {
    type Error = C3dParseError;
    fn try_from(data: ParameterData) -> Result<Self, Self::Error> {
        match data {
            ParameterData::Char(data, dimensions) => {
                if dimensions.len() == 2 && dimensions[1] == 4 {
                    let mut chars = Vec::new();
                    for row in 0..data.len() % 4 {
                        let mut chars_row = [' '; 4];
                        for i in 0..4 {
                            chars_row[i] = data[row * 4 + i];
                        }
                        chars.push(chars_row);
                    }
                    Ok(chars)
                } else {
                    Err(C3dParseError::InvalidData("Vec<[char; 4]>".to_string()))
                }
            }
            _ => Err(C3dParseError::InvalidData("Vec<[char; 4]>".to_string())),
        }
    }
}

fn get(
    raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    group_name: &str,
    parameter_name: &str,
) -> Option<ParameterData> {
    let group = raw_parameters.get(group_name)?;
    let parameter = group.get(parameter_name)?;
    Some(parameter.0.clone())
}

fn get_or_err(
    raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    group_name: &str,
    parameter_name: &str,
) -> Result<ParameterData, C3dParseError> {
    let group = raw_parameters
        .get(group_name)
        .ok_or(C3dParseError::GroupNotFound(group_name.to_owned()))?;
    let parameter = group
        .get(parameter_name)
        .ok_or(C3dParseError::ParameterNotFound(
            group_name.to_owned(),
            parameter_name.to_owned(),
        ))?;
    Ok(parameter.0.clone())
}

fn get_float(
    raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    group_name: &str,
    parameter_name: &str,
) -> Option<f32> {
    let group = raw_parameters.get(group_name)?;
    let parameter = group.get(parameter_name)?;
    match &parameter.0 {
        ParameterData::Float(data, dimensions) => {
            if data.len() == 1 {
                Some(data[0])
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn get_float_vec(
    raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    group_name: &str,
    parameter_name: &str,
) -> Option<Vec<f32>> {
    let group = raw_parameters.get(group_name)?;
    let parameter = group.get(parameter_name)?;
    match &parameter.0 {
        ParameterData::Float(data, dimensions) => {
            if dimensions.len() == 1 {
                Some(data.into_iter().map(|&x| x as f32).collect())
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn get_float_array2(
    raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    group_name: &str,
    parameter_name: &str,
) -> Option<DMatrix<f32>> {
    let group = raw_parameters.get(group_name)?;
    let parameter = group.get(parameter_name)?;
    match &parameter.0 {
        ParameterData::Float(data, dimensions) => {
            if dimensions.len() == 2 {
                let array = DMatrix::from_iterator(
                    dimensions[0],
                    dimensions[1],
                    data.into_iter().map(|&x| x as f32),
                );
                Some(array)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn get_cal_matrix_vector(
    raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    group_name: &str,
    parameter_name: &str,
) -> Option<Vec<Matrix6<f32>>> {
    let group = raw_parameters.get(group_name)?;
    let parameter = group.get(parameter_name)?;
    match &parameter.0 {
        ParameterData::Float(data, dimensions) => {
            if dimensions.len() == 3 {
                let mut array = Vec::new();
                let data_array: Vec<f32> = data.into_iter().map(|&x| x as f32).collect();
                for platform in 0..data.len() % 36 {
                    array.push(Matrix6::from_vec(
                        data_array[platform * 36..(platform + 1) * 36].into(),
                    ));
                }
                Some(array)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn get_signed_integer(
    raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    group_name: &str,
    parameter_name: &str,
) -> Option<i16> {
    let group = raw_parameters.get(group_name)?;
    let parameter = group.get(parameter_name)?;
    match &parameter.0 {
        ParameterData::Integer(data, dimensions) => {
            if data.len() == 1 {
                Some(data[0])
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn get_signed_integer_vec(
    raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    group_name: &str,
    parameter_name: &str,
) -> Option<Vec<i16>> {
    let group = raw_parameters.get(group_name)?;
    let parameter = group.get(parameter_name)?;
    match &parameter.0 {
        ParameterData::Integer(data, dimensions) => {
            if dimensions.len() == 1 {
                Some(data.into_iter().map(|&x| x as i16).collect())
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn get_signed_integer_array2(
    raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    group_name: &str,
    parameter_name: &str,
) -> Option<DMatrix<i16>> {
    let group = raw_parameters.get(group_name)?;
    let parameter = group.get(parameter_name)?;
    match &parameter.0 {
        ParameterData::Integer(data, dimensions) => {
            if dimensions.len() == 2 {
                let array = DMatrix::from_iterator(
                    dimensions[0],
                    dimensions[1],
                    data.into_iter().map(|&x| x as i16),
                );
                Some(array)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn get_string(
    raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    group_name: &str,
    parameter_name: &str,
) -> Option<String> {
    let group = raw_parameters.get(group_name)?;
    let parameter = group.get(parameter_name)?;
    match &parameter.0 {
        ParameterData::Char(data, dimensions) => {
            if dimensions.len() == 1 {
                let mut string = String::new();
                for c in data {
                    string.push(*c as char);
                }
                Some(string)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn get_string_vec(
    raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    group_name: &str,
    parameter_name: &str,
) -> Option<Vec<String>> {
    let group = raw_parameters.get(group_name)?;
    let parameter = group.get(parameter_name)?;
    match &parameter.0 {
        ParameterData::Char(data, dimensions) => {
            if dimensions.len() == 2 {
                let mut strings = Vec::new();
                for column in 0..dimensions[1] {
                    let mut string = String::new();
                    for c in 0..dimensions[0] {
                        string.push(data[column * dimensions[0] + c] as char);
                    }
                    strings.push(string);
                }
                Some(strings)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn get_char_pair(
    raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    group_name: &str,
    parameter_name: &str,
) -> Option<[char; 2]> {
    let group = raw_parameters.get(group_name)?;
    let parameter = group.get(parameter_name)?;
    match &parameter.0 {
        ParameterData::Char(data, dimensions) => {
            if dimensions.len() == 1 {
                let mut chars = [0 as char; 2];
                chars[0] = data[0] as char;
                chars[1] = data[1] as char;
                Some(chars)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn get_char_quad(
    raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    group_name: &str,
    parameter_name: &str,
) -> Option<[char; 4]> {
    let group = raw_parameters.get(group_name)?;
    let parameter = group.get(parameter_name)?;
    match &parameter.0 {
        ParameterData::Char(data, dimensions) => {
            if data.len() == 4 {
                let mut chars = [0 as char; 4];
                chars[0] = data[0] as char;
                chars[1] = data[1] as char;
                chars[2] = data[2] as char;
                chars[3] = data[3] as char;
                Some(chars)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn get_times_array(
    raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    group_name: &str,
    parameter_name: &str,
) -> Option<Vec<[f32; 2]>> {
    let group = raw_parameters.get(group_name)?;
    let parameter = group.get(parameter_name)?;
    match &parameter.0 {
        ParameterData::Float(data, dimensions) => {
            if dimensions.len() == 2 && data.len() > 1 {
                let mut times = Vec::new();
                for row in 0..data.len() % 2 {
                    let mut time = [0.0; 2];
                    time[0] = data[row * 2];
                    time[1] = data[row * 2 + 1];
                    times.push(time);
                }
                Some(times)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn get_colour_array(
    raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    group_name: &str,
    parameter_name: &str,
) -> Option<Vec<[u8; 3]>> {
    let group = raw_parameters.get(group_name)?;
    let parameter = group.get(parameter_name)?;
    match &parameter.0 {
        ParameterData::Byte(data, dimensions) => {
            if dimensions.len() == 2 {
                let mut colours = Vec::new();
                for row in 0..data.len() % 3 {
                    let mut colour = [0; 3];
                    colour[0] = data[row * 3];
                    colour[1] = data[row * 3 + 1];
                    colour[2] = data[row * 3 + 2];
                    colours.push(colour);
                }
                Some(colours)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DataType {
    Char,
    Byte,
    Integer,
    Float,
}

impl From<DataType> for usize {
    fn from(data_type: DataType) -> Self {
        match data_type {
            DataType::Char => 1,
            DataType::Byte => 1,
            DataType::Integer => 2,
            DataType::Float => 4,
        }
    }
}

impl TryFrom<i8> for DataType {
    type Error = C3dParseError;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            -1 => Ok(DataType::Char),
            1 => Ok(DataType::Byte),
            2 => Ok(DataType::Integer),
            4 => Ok(DataType::Float),
            _ => Err(C3dParseError::InvalidDataType),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterData {
    Char(DVector<char>, Vec<usize>),
    Byte(DVector<u8>, Vec<usize>),
    Integer(DVector<i16>, Vec<usize>),
    Float(DVector<f32>, Vec<usize>),
}

impl ParameterData {
    fn new(
        data: &[u8],
        data_type: DataType,
        dimensions: &[usize],
        processor: &Processor,
    ) -> Result<Self, C3dParseError> {
        if data.len() % usize::from(data_type) != 0 {
            return Err(C3dParseError::InvalidParameterData);
        }
        if dimensions.iter().product::<usize>() != data.len() / usize::from(data_type) {
            return Err(C3dParseError::InvalidParameterData);
        }
        let mut dimensions = dimensions
            .iter()
            .map(|&x| x as usize)
            .collect::<Vec<usize>>();
        if dimensions.len() == 0 {
            dimensions.push(1);
        }
        let array = match data_type {
            DataType::Char => {
                let array = DVector::from_iterator(data.len(), data.iter().map(|&x| x as char));
                ParameterData::Char(array, dimensions)
            }
            DataType::Byte => {
                let array = DVector::from_row_slice(data);
                ParameterData::Byte(array, dimensions)
            }
            DataType::Integer => {
                let array = DVector::from_iterator(
                    data.len() / 2,
                    data.chunks(2).map(|x| processor.i16(x.try_into().unwrap())),
                );
                ParameterData::Integer(array, dimensions)
            }
            DataType::Float => {
                let array = DVector::from_iterator(
                    data.len() / 4,
                    data.chunks(4).map(|x| processor.f32(x.try_into().unwrap())),
                );
                ParameterData::Float(array, dimensions)
            }
        };

        Ok(array)
    }
}

#[derive(Debug, Clone)]
struct Group {
    id: i8,
    name: String,
    description: String,
}

#[derive(Debug, Clone)]
struct Parameter {
    group_id: i8,
    name: String,
    pub data: ParameterData,
    description: String,
}

fn parse_parameter_blocks(
    parameter_blocks: &Vec<u8>,
    processor: &Processor,
) -> Result<
    (
        HashMap<String, HashMap<String, (ParameterData, String)>>,
        HashMap<String, String>,
    ),
    C3dParseError,
> {
    if parameter_blocks.len() < 512 {
        return Err(C3dParseError::InvalidParameterStartBlock);
    }

    let mut groups: Vec<Group> = Vec::new();
    let mut parameters: Vec<Parameter> = Vec::new();

    let mut index = 4;

    while index != 0 {
        index = parse_next_group_or_parameter(
            &parameter_blocks,
            index,
            &mut groups,
            &mut parameters,
            processor,
        )?;
    }
    let mut groups_map = HashMap::new();
    let mut group_descriptions = HashMap::new();
    let mut temp_group_id_to_name = HashMap::new();
    for group in groups {
        temp_group_id_to_name.insert(group.id, group.name.clone());
        groups_map.insert(group.name.clone(), HashMap::new());
        group_descriptions.insert(group.name, group.description);
    }
    for parameter in parameters {
        let group_name = match temp_group_id_to_name.contains_key(&parameter.group_id) {
            true => temp_group_id_to_name
                .get(&parameter.group_id)
                .unwrap()
                .clone(),
            false => {
                temp_group_id_to_name.insert(parameter.group_id, parameter.group_id.to_string());
                groups_map.insert(parameter.group_id.to_string(), HashMap::new());
                parameter.group_id.to_string()
            }
        };
        groups_map
            .get_mut(&group_name)
            .ok_or(C3dParseError::InvalidGroupId)?
            .insert(
                parameter.name.clone(),
                (parameter.data, parameter.description),
            );
    }
    Ok((groups_map, group_descriptions))
}

fn parse_next_group_or_parameter(
    parameter_blocks: &Vec<u8>,
    index: usize,
    groups: &mut Vec<Group>,
    parameters: &mut Vec<Parameter>,
    processor: &Processor,
) -> Result<usize, C3dParseError> {
    if index + 1 >= parameter_blocks.len() {
        return Ok(0);
        //return Err(C3dParseError::InvalidNextParameter);
    }
    let group_id = parameter_blocks[index + 1] as i8;

    if group_id == 0 {
        return Ok(0);
    } else if group_id < 0 {
        let (group, next_index) = parse_group(&parameter_blocks, index, processor)?;
        groups.push(group);
        Ok(next_index as usize)
    } else {
        let (parameter, next_index) = parse_parameter(&parameter_blocks, index, processor)?;
        parameters.push(parameter);
        Ok(next_index as usize)
    }
}

fn parse_group(
    parameter_blocks: &Vec<u8>,
    index: usize,
    processor: &Processor,
) -> Result<(Group, usize), C3dParseError> {
    let mut i = index;
    let num_chars_in_name = parameter_blocks[i] as i8;
    i += 1;
    let id = (parameter_blocks[i] as i8).abs();
    i += 1;
    let name = parse_group_name(&parameter_blocks, i, num_chars_in_name)?;
    i += num_chars_in_name.abs() as usize;
    let next_group_index_bytes = &parameter_blocks[i..i + 2];
    let next_group_index =
        processor.u16(next_group_index_bytes.try_into().unwrap()) as usize + i as usize;
    i += 2;
    let num_chars_in_description = parameter_blocks[i];
    i += 1;
    let description = parse_description(&parameter_blocks, i, num_chars_in_description)?;

    Ok((
        Group {
            id,
            name,
            description,
        },
        next_group_index,
    ))
}

fn parse_group_name(
    parameter_blocks: &Vec<u8>,
    index: usize,
    num_chars_in_name: i8,
) -> Result<String, C3dParseError> {
    let mut group_name = String::new();

    for i in 0..num_chars_in_name.abs() {
        group_name.push(parameter_blocks[index + i as usize] as char);
    }

    Ok(group_name)
}

fn parse_description(
    parameter_blocks: &Vec<u8>,
    index: usize,
    num_chars_in_description: u8,
) -> Result<String, C3dParseError> {
    let mut description = String::new();

    for i in 0..num_chars_in_description {
        description.push(parameter_blocks[index + i as usize] as char);
    }

    Ok(description)
}

fn parse_parameter(
    parameter_blocks: &Vec<u8>,
    index: usize,
    processor: &Processor,
) -> Result<(Parameter, usize), C3dParseError> {
    let mut i = index;
    let num_chars_in_name = parameter_blocks[i] as i8;
    i += 1;
    let group_id = parameter_blocks[i] as i8;
    i += 1;
    let name = parse_parameter_name(&parameter_blocks, i, num_chars_in_name)?;
    i += num_chars_in_name.abs() as usize;
    let next_index_bytes = &parameter_blocks[i..i + 2];
    let next_index = processor.u16(next_index_bytes.try_into().unwrap()) as usize + i as usize;
    i += 2;
    let data_type = DataType::try_from(parameter_blocks[i] as i8)?;
    i += 1;
    let num_dimensions = parameter_blocks[i];
    i += 1;
    let dimensions = parse_dimensions(&parameter_blocks, i, num_dimensions)?;
    i += num_dimensions as usize;
    let (data, data_byte_size) =
        parse_data(&parameter_blocks, i, &dimensions, data_type, processor)?;
    i += data_byte_size;
    let num_chars_in_description = parameter_blocks[i];
    i += 1;
    let description = parse_description(&parameter_blocks, i, num_chars_in_description)?;

    Ok((
        Parameter {
            group_id,
            name,
            data,
            description,
        },
        next_index,
    ))
}

fn parse_parameter_name(
    parameter_blocks: &[u8],
    index: usize,
    num_chars_in_name: i8,
) -> Result<String, C3dParseError> {
    let mut parameter_name = String::new();

    for i in 0..num_chars_in_name.abs() {
        parameter_name.push(parameter_blocks[index + i as usize] as char);
    }

    Ok(parameter_name)
}

fn parse_dimensions(
    parameter_blocks: &[u8],
    index: usize,
    num_dimensions: u8,
) -> Result<Vec<u8>, C3dParseError> {
    let mut dimensions = Vec::new();

    for i in 0..num_dimensions {
        dimensions.push(parameter_blocks[index + i as usize]);
    }

    Ok(dimensions)
}

fn parse_data(
    parameter_blocks: &Vec<u8>,
    index: usize,
    dimensions: &Vec<u8>,
    data_type: DataType,
    processor: &Processor,
) -> Result<(ParameterData, usize), C3dParseError> {
    let dimensions_product = &dimensions
        .clone()
        .iter()
        .map(|x| *x as usize)
        .product::<usize>();

    let data_byte_size = dimensions_product * usize::from(data_type);

    if index + data_byte_size > parameter_blocks.len() {
        return Err(C3dParseError::InvalidParameterData);
    }

    let bytes: &[u8] = &parameter_blocks[index..index + data_byte_size];
    let dimensions: &[usize] = &dimensions
        .iter()
        .map(|x| *x as usize)
        .collect::<Vec<usize>>();

    Ok((
        ParameterData::new(bytes, data_type, dimensions, processor)?,
        data_byte_size,
    ))
}
