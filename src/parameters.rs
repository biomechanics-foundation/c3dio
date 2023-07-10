use std::collections::HashMap;

use crate::processor::Processor;
use crate::C3dParseError;
use ndarray::{Array, Array2, Array3, ArrayView, Ix2, Ix3, IxDyn, Order};

#[derive(Debug, Clone)]
pub struct Parameters {
    pub group_descriptions: HashMap<String, String>,
    pub raw_parameters: HashMap<String, HashMap<String, (ParameterData, String)>>,
    pub required_parameters: RequiredParameters,
    pub additional_parameters: AdditionalParameters,
    pub application_parameters: ApplicationParameters,
}

impl Parameters {
    pub fn new() -> Parameters {
        Parameters {
            group_descriptions: HashMap::new(),
            raw_parameters: HashMap::new(),
            required_parameters: RequiredParameters::new(),
            additional_parameters: AdditionalParameters::new(),
            application_parameters: ApplicationParameters::new(),
        }
    }

    pub fn parse_parameter_blocks(
        parameter_blocks: &Vec<u8>,
        processor: &Processor,
    ) -> Result<Self, C3dParseError> {
        let (raw_parameters, group_descriptions) =
            parse_parameter_blocks(parameter_blocks, processor)?;
        let required_parameters = RequiredParameters::from_raw(&raw_parameters)?;
        let additional_parameters = AdditionalParameters::from_raw(&raw_parameters);
        let application_parameters = ApplicationParameters::from_raw(&raw_parameters);
        Ok(Parameters {
            group_descriptions,
            raw_parameters,
            required_parameters,
            additional_parameters,
            application_parameters,
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
            && self.required_parameters == other.required_parameters
            && self.additional_parameters == other.additional_parameters
            && self.application_parameters == other.application_parameters
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnalogOffset {
    Signed(Vec<i16>),
    Unsigned(Vec<u16>),
}

impl AnalogOffset {
    fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
        format: &AnalogFormat,
    ) -> Result<AnalogOffset, C3dParseError> {
        let offset = get_signed_integer_vec(raw_parameters, "ANALOG", "OFFSET");
        let offset = match offset {
            Some(offset) => offset,
            None => {
                return Ok(AnalogOffset::Signed(Vec::new()))
            },
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

#[derive(Debug, Clone, PartialEq)]
pub struct RequiredParameters {
    pub point: RequiredPointParameters,
    pub analog: RequiredAnalogParameters,
    pub force_platform: RequiredForcePlatformParameters,
}

impl RequiredParameters {
    pub fn new() -> Self {
        RequiredParameters {
            point: RequiredPointParameters::new(),
            analog: RequiredAnalogParameters::new(),
            force_platform: RequiredForcePlatformParameters::new(),
        }
    }

    pub fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    ) -> Result<Self, C3dParseError> {
        let required_parameters = RequiredParameters {
            point: RequiredPointParameters::from_raw(raw_parameters)?,
            analog: RequiredAnalogParameters::from_raw(raw_parameters)?,
            force_platform: RequiredForcePlatformParameters::from_raw(raw_parameters)?,
        };
        Ok(required_parameters)
    }
}

#[derive(Debug, Clone)]
pub struct RequiredPointParameters {
    pub used: u16,
    pub scale: f32,
    pub rate: f32,
    pub data_start: u16,
    pub frames: u16,
    pub labels: Vec<String>,
    pub descriptions: Vec<String>,
    pub units: [char; 4],
}

impl PartialEq for RequiredPointParameters {
    fn eq(&self, other: &Self) -> bool {
        self.used == other.used
//            && self.scale == other.scale
            && self.rate == other.rate
//            && self.data_start == other.data_start
            && self.frames == other.frames
            && self.labels == other.labels
            && self.descriptions == other.descriptions
            && self.units == other.units
    }
}

impl RequiredPointParameters {
    pub fn new() -> Self {
        RequiredPointParameters {
            used: 0,
            scale: 0.0,
            rate: 0.0,
            data_start: 0,
            frames: 0,
            labels: Vec::new(),
            descriptions: Vec::new(),
            units: [' '; 4],
        }
    }

    pub fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    ) -> Result<Self, C3dParseError> {
        let used = get_signed_integer(raw_parameters, "POINT", "USED");
        if used.is_none() || used.unwrap() == 0 {
            return Ok(RequiredPointParameters::new());
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
        let required_point_parameters = RequiredPointParameters {
            used: get_or_err(raw_parameters, "POINT", "USED")?.try_into()?,
            scale: get_or_err(raw_parameters, "POINT", "SCALE")?.try_into()?,
            rate: get_float(raw_parameters, "POINT", "RATE").unwrap_or(0.0),
            data_start: get_or_err(raw_parameters, "POINT", "DATA_START")?.try_into()?,
            frames: frames.unwrap() as u16,
            labels: get_string_vec(raw_parameters, "POINT", "LABELS").unwrap_or(Vec::new()),
            descriptions: get_string_vec(raw_parameters, "POINT", "DESCRIPTIONS")
                .unwrap_or(Vec::new()),
            units: get_char_quad(raw_parameters, "POINT", "UNITS").unwrap_or([' '; 4]),
        };
        Ok(required_point_parameters)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnalogFormat {
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

#[derive(Debug, Clone, PartialEq)]
pub struct RequiredAnalogParameters {
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
}

impl RequiredAnalogParameters {
    pub fn new() -> Self {
        RequiredAnalogParameters {
            used: 0,
            labels: Vec::new(),
            descriptions: Vec::new(),
            gen_scale: 1.0,
            offset: AnalogOffset::Signed(Vec::new()),
            units: Vec::new(),
            scale: Vec::new(),
            rate: 0.0,
            format: AnalogFormat::Signed,
            bits: 0,
        }
    }

    pub fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    ) -> Result<Self, C3dParseError> {
        let used = get_signed_integer(raw_parameters, "ANALOG", "USED");
        if used.is_none() || used.unwrap() == 0 {
            return Ok(RequiredAnalogParameters::new());
        }
        let format = AnalogFormat::from_raw(raw_parameters);
        let offset = AnalogOffset::from_raw(raw_parameters, &format)?;
        let required_analog_parameters = RequiredAnalogParameters {
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
        };
        Ok(required_analog_parameters)
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
            ParameterData::Float(data) => {
                if data.ndim() == 3 {
                    for i in 0..data.shape()[2] {
                        let mut corner = [0.0; 12];
                        for j in 0..data.shape()[0] {
                            for k in 0..data.shape()[1] {
                                corner[j * data.shape()[1] + k] = data[[j, k, i]];
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
            ParameterData::Float(data) => {
                if data.ndim() == 2 {
                    for i in 0..data.shape()[1] {
                        let mut origin_point = [0.0; 3];
                        for j in 0..data.shape()[0] {
                            origin_point[j] = data[[j, i]];
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

#[derive(Debug, Clone, PartialEq)]
pub struct RequiredForcePlatformParameters {
    pub used: u16,
    pub plate_type: Option<Vec<ForcePlatformType>>,
    pub zero: Option<[u16; 2]>,
    pub corners: Option<ForcePlatformCorners>,
    pub origin: Option<ForcePlatformOrigin>,
    pub channel: Option<Array2<i16>>,
}

impl RequiredForcePlatformParameters {
    pub fn new() -> Self {
        RequiredForcePlatformParameters {
            used: 0,
            plate_type: None,
            zero: None,
            corners: None,
            origin: None,
            channel: None,
        }
    }

    pub fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    ) -> Result<Self, C3dParseError> {
        let used = get_signed_integer(raw_parameters, "FORCE_PLATFORM", "USED");
        if used.is_none() || used.unwrap() == 0 {
            return Ok(RequiredForcePlatformParameters::new());
        }
        let plate_type = ForcePlatformType::from_raw(raw_parameters)?;
        let corners = ForcePlatformCorners::from_raw(raw_parameters)?;
        let origin = ForcePlatformOrigin::from_raw(raw_parameters)?;
        let zero = get_or_err(raw_parameters, "FORCE_PLATFORM", "ZERO")?.try_into()?;
        let channel = get_signed_integer_array2(raw_parameters, "FORCE_PLATFORM", "CHANNEL");
        let required_force_platform_parameters = RequiredForcePlatformParameters {
            used: used.unwrap() as u16,
            plate_type: Some(plate_type),
            zero: Some(zero),
            corners: Some(corners),
            origin: Some(origin),
            channel,
        };
        Ok(required_force_platform_parameters)
    }
}

impl TryFrom<ParameterData> for u16 {
    type Error = C3dParseError;
    fn try_from(data: ParameterData) -> Result<Self, Self::Error> {
        match data {
            ParameterData::Integer(data) => {
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
            ParameterData::Integer(data) => {
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
            ParameterData::Integer(data) => {
                if data.ndim() == 1 {
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
            ParameterData::Integer(data) => {
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
            ParameterData::Integer(data) => {
                if data.ndim() == 1 {
                    Ok(data.to_owned().into_raw_vec())
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
            ParameterData::Float(data) => {
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
            ParameterData::Float(data) => {
                if data.ndim() == 1 {
                    Ok(data.to_owned().into_raw_vec())
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
            ParameterData::Char(data) => {
                let mut string = String::new();
                for c in data {
                    string.push(c);
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
            ParameterData::Char(data) => {
                if data.ndim() == 2 || data.ndim() == 1 {
                    let mut strings = Vec::new();
                    for row in data.rows() {
                        let mut string = String::new();
                        for c in row {
                            string.push(*c);
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
            ParameterData::Char(data) => {
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
            ParameterData::Char(data) => {
                if data.ndim() == 2 {
                    let mut chars = Vec::new();
                    for row in data.rows() {
                        if row.len() == 4 {
                            let mut chars_row = [' '; 4];
                            for (i, c) in row.iter().enumerate() {
                                chars_row[i] = *c;
                            }
                            chars.push(chars_row);
                        } else {
                            return Err(C3dParseError::InvalidData("Vec<[char; 4]>".to_string()));
                        }
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

#[derive(Debug, Clone, PartialEq)]
pub struct AdditionalParameters {
    pub point_long_frames: Option<f32>,
    pub force_platform_cal_matrix: Option<Array3<f32>>,
    pub trial_actual_start_field: Option<u16>,
    pub trial_actual_end_field: Option<u16>,
    pub trial_camera_rate: Option<f32>,
    pub event_used: Option<i16>,
    pub event_contexts: Option<Vec<String>>,
    pub event_labels: Option<Vec<String>>,
    pub event_descriptions: Option<Vec<String>>,
    pub event_times: Option<Vec<[f32; 2]>>,
    pub event_subjects: Option<Vec<String>>,
    pub event_icon_ids: Option<Vec<i16>>,
    pub event_generic_flags: Option<Vec<i16>>,
    pub event_context_used: Option<i16>,
    pub event_context_icon_ids: Option<Vec<u16>>,
    pub event_context_labels: Option<Vec<String>>,
    pub event_context_descriptions: Option<Vec<String>>,
    pub event_context_colours: Option<Vec<[u8; 3]>>,
}

impl AdditionalParameters {
    pub fn new() -> Self {
        AdditionalParameters {
            point_long_frames: None,
            force_platform_cal_matrix: None,
            trial_actual_start_field: None,
            trial_actual_end_field: None,
            trial_camera_rate: None,
            event_used: None,
            event_contexts: None,
            event_labels: None,
            event_descriptions: None,
            event_times: None,
            event_subjects: None,
            event_icon_ids: None,
            event_generic_flags: None,
            event_context_used: None,
            event_context_icon_ids: None,
            event_context_labels: None,
            event_context_descriptions: None,
            event_context_colours: None,
        }
    }

    pub fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    ) -> Self {
        AdditionalParameters {
            point_long_frames: get_float(raw_parameters, "POINT", "LONG_FRAMES"),
            force_platform_cal_matrix: get_float_array3(
                raw_parameters,
                "FORCE_PLATFORM",
                "CAL_MATRIX",
            ),
            trial_actual_start_field: get_signed_integer(
                raw_parameters,
                "TRIAL",
                "ACTUAL_START_FIELD",
            )
            .map(|x| x as u16),
            trial_actual_end_field: get_signed_integer(raw_parameters, "TRIAL", "ACTUAL_END_FIELD")
                .map(|x| x as u16),
            trial_camera_rate: get_float(raw_parameters, "TRIAL", "CAMERA_RATE"),
            event_used: get_signed_integer(raw_parameters, "EVENT", "USED"),
            event_contexts: get_string_vec(raw_parameters, "EVENT", "CONTEXTS"),
            event_labels: get_string_vec(raw_parameters, "EVENT", "LABELS"),
            event_descriptions: get_string_vec(raw_parameters, "EVENT", "DESCRIPTIONS"),
            event_times: get_times_array(raw_parameters, "EVENT", "TIMES"),
            event_subjects: get_string_vec(raw_parameters, "EVENT", "SUBJECTS"),
            event_icon_ids: get_signed_integer_vec(raw_parameters, "EVENT", "ICON_IDS"),
            event_generic_flags: get_signed_integer_vec(raw_parameters, "EVENT", "GENERIC_FLAGS"),
            event_context_used: get_signed_integer(raw_parameters, "EVENT_CONTEXT", "USED"),
            event_context_icon_ids: get_signed_integer_vec(
                raw_parameters,
                "EVENT_CONTEXT",
                "ICON_IDS",
            )
            .map(|x| x.iter().map(|&x| x as u16).collect()),
            event_context_labels: get_string_vec(raw_parameters, "EVENT_CONTEXT", "LABELS"),
            event_context_descriptions: get_string_vec(
                raw_parameters,
                "EVENT_CONTEXT",
                "DESCRIPTIONS",
            ),
            event_context_colours: get_colour_array(raw_parameters, "EVENT_CONTEXT", "COLOURS"),
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
pub struct ApplicationParameters {
    pub analog_gain: Option<Vec<i16>>,
    pub manufacturer_company: Option<String>,
    pub manufacturer_software: Option<String>,
    pub manufacturer_version: Option<ManufacturerVersion>,
    pub manufacturer_edited: Option<Vec<String>>,
    pub point_x_screen: Option<[char; 2]>,
    pub point_y_screen: Option<[char; 2]>,
    pub seg_marker_diameter: Option<f32>,
    pub seg_data_limits: Option<Array2<f32>>,
    pub seg_acc_factor: Option<f32>,
    pub seg_noise_factor: Option<f32>,
    pub seg_residual_error_factor: Option<f32>,
    pub seg_intersection_limit: Option<f32>,
}

impl ApplicationParameters {
    pub fn new() -> Self {
        ApplicationParameters {
            analog_gain: None,
            manufacturer_company: None,
            manufacturer_software: None,
            manufacturer_version: None,
            manufacturer_edited: None,
            point_x_screen: None,
            point_y_screen: None,
            seg_marker_diameter: None,
            seg_data_limits: None,
            seg_acc_factor: None,
            seg_noise_factor: None,
            seg_residual_error_factor: None,
            seg_intersection_limit: None,
        }
    }

    pub fn from_raw(
        raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    ) -> Self {
        ApplicationParameters {
            analog_gain: get_signed_integer_vec(raw_parameters, "ANALOG", "GAIN"),
            manufacturer_company: get_string(raw_parameters, "MANUFACTURER", "COMPANY"),
            manufacturer_software: get_string(raw_parameters, "MANUFACTURER", "SOFTWARE"),
            manufacturer_version: None, // TODO
            manufacturer_edited: get_string_vec(raw_parameters, "MANUFACTURER", "EDITED"),
            point_x_screen: get_char_pair(raw_parameters, "POINT", "X_SCREEN"),
            point_y_screen: get_char_pair(raw_parameters, "POINT", "Y_SCREEN"),
            seg_marker_diameter: get_float(raw_parameters, "SEG", "MARKER_DIAMETER"),
            seg_data_limits: get_float_array2(raw_parameters, "SEG", "DATA_LIMITS"),
            seg_acc_factor: get_float(raw_parameters, "SEG", "ACC_FACTOR"),
            seg_noise_factor: get_float(raw_parameters, "SEG", "NOISE_FACTOR"),
            seg_residual_error_factor: get_float(raw_parameters, "SEG", "RESIDUAL_ERROR_FACTOR"),
            seg_intersection_limit: get_float(raw_parameters, "SEG", "INTERSECTION_LIMIT"),
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
        ParameterData::Float(data) => {
            if data.len() == 1 {
                data.first().cloned()
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
        ParameterData::Float(data) => {
            if data.ndim() == 1 {
                Some(data.to_owned().into_raw_vec())
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
) -> Option<Array2<f32>> {
    let group = raw_parameters.get(group_name)?;
    let parameter = group.get(parameter_name)?;
    match &parameter.0 {
        ParameterData::Float(data) => {
            if data.ndim() == 2 {
                let array = data.clone().into_dimensionality::<Ix2>();
                if array.is_ok() {
                    Some(array.unwrap().to_owned())
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn get_float_array3(
    raw_parameters: &HashMap<String, HashMap<String, (ParameterData, String)>>,
    group_name: &str,
    parameter_name: &str,
) -> Option<Array3<f32>> {
    let group = raw_parameters.get(group_name)?;
    let parameter = group.get(parameter_name)?;
    match &parameter.0 {
        ParameterData::Float(data) => {
            if data.ndim() == 2 {
                let array = data.clone().into_dimensionality::<Ix3>();
                if array.is_ok() {
                    Some(array.unwrap().to_owned())
                } else {
                    None
                }
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
        ParameterData::Integer(data) => {
            if data.len() == 1 {
                data.first().cloned()
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
        ParameterData::Integer(data) => {
            if data.ndim() == 1 {
                Some(data.to_owned().into_raw_vec())
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
) -> Option<Array2<i16>> {
    let group = raw_parameters.get(group_name)?;
    let parameter = group.get(parameter_name)?;
    match &parameter.0 {
        ParameterData::Integer(data) => {
            if data.ndim() == 2 {
                let array = data.clone().into_dimensionality::<Ix2>();
                if array.is_ok() {
                    Some(array.unwrap().to_owned())
                } else {
                    None
                }
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
        ParameterData::Char(data) => {
            if data.ndim() == 1 {
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
        ParameterData::Char(data) => {
            if data.ndim() == 2 {
                let mut strings = Vec::new();
                for column in data.columns() {
                    let mut string = String::new();
                    for c in column {
                        string.push(*c as char);
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
        ParameterData::Char(data) => {
            if data.ndim() == 1 {
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
        ParameterData::Char(data) => {
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
        ParameterData::Float(data) => {
            if data.ndim() == 2 && data.len() > 1 {
                let mut times = Vec::new();
                for row in data.rows() {
                    let mut time = [0.0; 2];
                    time[0] = row[0];
                    time[1] = row[1];
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
        ParameterData::Byte(data) => {
            if data.ndim() == 2 {
                let mut colours = Vec::new();
                for row in data.rows() {
                    let mut colour = [0; 3];
                    colour[0] = row[0];
                    colour[1] = row[1];
                    colour[2] = row[2];
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
    Char(Array<char, IxDyn>),
    Byte(Array<u8, IxDyn>),
    Integer(Array<i16, IxDyn>),
    Float(Array<f32, IxDyn>),
}

pub enum CharParameterData {
    Char(Array<char, IxDyn>),
}

impl From<Array<char, IxDyn>> for ParameterData {
    fn from(array: Array<char, IxDyn>) -> Self {
        ParameterData::Char(array)
    }
}

impl From<Array<u8, IxDyn>> for ParameterData {
    fn from(array: Array<u8, IxDyn>) -> Self {
        ParameterData::Byte(array)
    }
}

impl From<Array<i16, IxDyn>> for ParameterData {
    fn from(array: Array<i16, IxDyn>) -> Self {
        ParameterData::Integer(array)
    }
}

impl From<Array<f32, IxDyn>> for ParameterData {
    fn from(array: Array<f32, IxDyn>) -> Self {
        ParameterData::Float(array)
    }
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
        let shape = IxDyn(dimensions.as_slice());
        let array = match data_type {
            DataType::Char => {
                let data = data.iter().map(|&x| x as char).collect::<Vec<char>>();
                let array = ArrayView::<char, IxDyn>::from_shape(
                    IxDyn(vec![data.len()].as_slice()),
                    data.as_slice(),
                )
                .unwrap();
                let array = array
                    .to_shape(((shape), Order::ColumnMajor))
                    .unwrap()
                    .to_owned();
                ParameterData::Char(array)
            }
            DataType::Byte => {
                let array =
                    ArrayView::<u8, IxDyn>::from_shape(IxDyn(vec![data.len()].as_slice()), data)
                        .unwrap();
                let array = array
                    .to_shape(((shape), Order::ColumnMajor))
                    .unwrap()
                    .to_owned();
                ParameterData::from(array)
            }
            DataType::Integer => {
                let data = data
                    .chunks(2)
                    .map(|x| processor.i16(x.try_into().unwrap()))
                    .collect::<Vec<i16>>();
                let array = ArrayView::<i16, IxDyn>::from_shape(
                    IxDyn(vec![data.len()].as_slice()),
                    data.as_slice(),
                )
                .unwrap();
                let array = array
                    .to_shape(((shape), Order::ColumnMajor))
                    .unwrap()
                    .to_owned();
                ParameterData::Integer(array)
            }
            DataType::Float => {
                let data = data
                    .chunks(4)
                    .map(|x| processor.f32(x.try_into().unwrap()))
                    .collect::<Vec<f32>>();
                let array = ArrayView::<f32, IxDyn>::from_shape(
                    IxDyn(vec![data.len()].as_slice()),
                    data.as_slice(),
                )
                .unwrap();
                let array = array
                    .to_shape(((shape), Order::ColumnMajor))
                    .unwrap()
                    .to_owned();
                ParameterData::Float(array)
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
