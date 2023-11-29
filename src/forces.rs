use crate::parameters::{Parameter, ParameterData, Parameters};
use crate::processor::Processor;
use crate::{C3dIoError, C3dParseError};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ForcePlatforms {
    pub force_platforms: Vec<ForcePlatform>,
    pub zero: [u16; 2],
}

impl Deref for ForcePlatforms {
    type Target = Vec<ForcePlatform>;

    fn deref(&self) -> &Self::Target {
        &self.force_platforms
    }
}

impl DerefMut for ForcePlatforms {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.force_platforms
    }
}

impl ToString for ForcePlatforms {
    fn to_string(&self) -> String {
        let mut s = String::new();
        s.push_str("Force Platforms:\n");
        s.push_str("Zero:\n");
        s.push_str(&format!("  {:?}\n", self.zero));
        s.push_str("Force Platforms:\n");
        for (i, force_platform) in self.force_platforms.iter().enumerate() {
            s.push_str(&format!("  {}: {:?}\n", i, force_platform));
        }
        s
    }
}

#[derive(Debug, Clone, Default)]
pub struct ForcePlatform {
    pub plate_type: ForcePlatformType,
    pub corners: ForcePlatformCorners,
    pub origin: ForcePlatformOrigin,
    pub channels: [u8; 8],
    pub cal_matrix: Option<[[f32; 6]; 6]>,
}

impl PartialEq for ForcePlatform {
    fn eq(&self, other: &Self) -> bool {
        self.plate_type == other.plate_type
            && self.corners == other.corners
            && self.origin == other.origin
            && self.channels == other.channels
            && self.cal_matrix == other.cal_matrix
    }
}

impl ToString for ForcePlatform {
    fn to_string(&self) -> String {
        let mut s = String::new();
        s.push_str("Plate Type:\n");
        s.push_str(&format!("  {:?}\n", self.plate_type));
        s.push_str("Corners:\n");
        for (i, corners) in self.corners.iter().enumerate() {
            s.push_str(&format!("  {}: {:?}\n", i, corners));
        }
        s.push_str("Origin:\n");
        s.push_str(&format!("  {:?}\n", self.origin));
        s.push_str("Channels:\n");
        s.push_str(&format!("  {:?}\n", self.channels));
        s.push_str("Cal Matrix:\n");
        s.push_str(&format!("  {:?}\n", self.cal_matrix));
        s
    }
}

impl ForcePlatforms {
    pub(crate) fn from_parameters(parameters: &mut Parameters) -> Result<Self, C3dParseError> {
        let used_parameter = parameters.remove("FORCE_PLATFORM", "USED");
        let used: Option<u16> = match used_parameter {
            None => None,
            Some(parameter) => Some(parameter.as_ref().try_into()?),
        };
        let mut is_none_or_zero = used.is_none();
        if !is_none_or_zero {
            let used = used.unwrap();
            is_none_or_zero = used == 0;
        }
        if is_none_or_zero {
            return Ok(ForcePlatforms::default());
        } else {
            let used = used.unwrap();
            let plate_type = ForcePlatformType::from_parameters(parameters, used)?;
            let corners = ForcePlatformCorners::from_parameters(parameters, used)?;
            let origin = ForcePlatformOrigin::from_parameters(parameters, used)?;
            let zero = parameters
                .remove_or_err("FORCE_PLATFORM", "ZERO")?
                .as_ref()
                .try_into()?;
            let contains_type_3 = plate_type.iter().any(|x| *x == ForcePlatformType::Type3);
            let channels = get_channels(parameters, used, contains_type_3)?;
            let cal_matrices = get_cal_matrix_vector(parameters, &plate_type)?;

            let mut force_platforms = Vec::new();
            for i in 0..used as usize {
                let mut force_platform = ForcePlatform::default();
                force_platform.plate_type = plate_type[i].clone();
                force_platform.corners = corners[i].clone();
                force_platform.origin = origin[i].clone();
                force_platform.channels = channels[i];
                force_platform.cal_matrix = cal_matrices[i].clone();
                force_platforms.push(force_platform);
            }
            let force_platform_parameters = ForcePlatforms {
                zero,
                force_platforms,
            };
            Ok(force_platform_parameters)
        }
    }

    pub fn len(&self) -> usize {
        self.force_platforms.len()
    }

    pub(crate) fn write(
        &self,
        processor: &Processor,
        group_names_to_ids: &HashMap<String, usize>,
    ) -> Result<Vec<u8>, C3dIoError> {
        let mut bytes = Vec::new();
        // "FORCE_PLATFORM", "USED"
        bytes.extend(Parameter::integer(self.force_platforms.len() as i16).write(
            processor,
            "USED".to_string(),
            group_names_to_ids["FORCE_PLATFORM"],
            false,
        )?);
        // "FORCE_PLATFORM", "CORNERS"
        if self.force_platforms.len() > 0 {
            let mut corners = Parameter::floats(
                self.force_platforms
                    .iter()
                    .map(|x| {
                        let mut out: Vec<f32> = Vec::new();
                        for corner in x.corners.clone().to_vec() {
                            out.extend(corner.to_vec());
                        }
                        out
                    })
                    .flatten()
                    .collect::<Vec<f32>>(),
            )?;
            corners.dimensions = vec![3, 4, self.force_platforms.len() as u8];
            bytes.extend(corners.write(
                processor,
                "CORNERS".to_string(),
                group_names_to_ids["FORCE_PLATFORM"],
                false,
            )?);
        }
        // "FORCE_PLATFORM", "ORIGIN"
        if self.force_platforms.len() > 0 {
            let mut origin = Parameter::floats(
                self.force_platforms
                    .iter()
                    .map(|x| x.origin.clone().to_vec())
                    .flatten()
                    .collect::<Vec<f32>>(),
            )?;
            origin.dimensions = vec![3, self.force_platforms.len() as u8];
            bytes.extend(origin.write(
                processor,
                "ORIGIN".to_string(),
                group_names_to_ids["FORCE_PLATFORM"],
                false,
            )?);
        }
        // "FORCE_PLATFORM", "CHANNEL"
        if self.force_platforms.len() > 0 {
            let mut channels = Parameter::integers(
                self.force_platforms
                    .iter()
                    .map(|x| {
                        x.channels
                            .clone()
                            .to_vec()
                            .iter()
                            .map(|y| y.clone() as i16)
                            .collect::<Vec<i16>>()
                    })
                    .flatten()
                    .collect::<Vec<i16>>(),
            )?;
            channels.dimensions = vec![8, self.force_platforms.len() as u8];
            bytes.extend(channels.write(
                processor,
                "CHANNEL".to_string(),
                group_names_to_ids["FORCE_PLATFORM"],
                false,
            )?);
        }
        // "FORCE_PLATFORM", "CAL_MATRIX"
        if self.force_platforms.len() > 0 {
            if self.force_platforms.iter().any(|x| x.cal_matrix.is_some()) {
                let mut cal_matrices = Parameter::floats(
                    self.force_platforms
                        .iter()
                        .map(|force_platform| {
                            if force_platform.cal_matrix.is_some() {
                                force_platform
                                    .cal_matrix
                                    .unwrap()
                                    .to_vec()
                                    .iter()
                                    .map(|x| {
                                        x.to_vec()
                                            .iter()
                                            .map(|y| y.clone() as f32)
                                            .collect::<Vec<f32>>()
                                    })
                                    .flatten()
                                    .collect::<Vec<f32>>()
                            } else {
                                let mut temp: Vec<f32> = Vec::new();
                                for _ in 0..36 {
                                    temp.push(0.0);
                                }
                                temp
                            }
                        })
                        .flatten()
                        .collect::<Vec<f32>>(),
                )?;
                cal_matrices.dimensions = vec![6, 6, self.force_platforms.len() as u8];
                bytes.extend(cal_matrices.write(
                    processor,
                    "CAL_MATRIX".to_string(),
                    group_names_to_ids["FORCE_PLATFORM"],
                    false,
                )?);
            }
        }
        // "FORCE_PLATFORM", "ZERO"
        bytes.extend(
            Parameter::integers(self.zero.to_vec().iter().map(|x| *x as i16).collect())?.write(
                processor,
                "ZERO".to_string(),
                group_names_to_ids["FORCE_PLATFORM"],
                false,
            )?,
        );
        // "FORCE_PLATFORM", "TYPE"
        if self.force_platforms.len() > 0 {
            bytes.extend(
                Parameter::integers(
                    self.force_platforms
                        .iter()
                        .map(|x| match x.plate_type {
                            ForcePlatformType::Type1 => 1,
                            ForcePlatformType::Type2 => 2,
                            ForcePlatformType::Type3 => 3,
                            ForcePlatformType::Type4 => 4,
                        })
                        .collect::<Vec<i16>>(),
                )?
                .write(
                    processor,
                    "TYPE".to_string(),
                    group_names_to_ids["FORCE_PLATFORM"],
                    false,
                )?,
            );
        }
        Ok(bytes)
    }

    pub(crate) fn force_from_analog(
        &self,
        analog: [f32; 8],
        force_platform: usize,
    ) -> Option<[f32; 3]> {
        if force_platform < self.force_platforms.len() {
            Some(
                self.force_platforms[force_platform]
                    .plate_type
                    .force_from_analog(analog),
            )
        } else {
            None
        }
    }

    pub(crate) fn center_of_pressure_from_analog(
        &self,
        analog: [f32; 8],
        force_platform: usize,
    ) -> Option<[f32; 2]> {
        if force_platform < self.force_platforms.len() {
            Some(
                self.force_platforms[force_platform]
                    .plate_type
                    .center_of_pressure_from_analog(analog),
            )
        } else {
            None
        }
    }

    pub fn origin(&self, force_platform: usize) -> Option<&ForcePlatformOrigin> {
        if force_platform < self.force_platforms.len() {
            Some(&self.force_platforms[force_platform].origin)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ForcePlatformType {
    Type1,
    #[default]
    Type2,
    Type3,
    Type4,
}

impl ForcePlatformType {
    fn from_parameters(
        parameters: &mut Parameters,
        used: u16,
    ) -> Result<Vec<ForcePlatformType>, C3dParseError> {
        let force_platform_type: Vec<i16> = parameters
            .remove_or_err("FORCE_PLATFORM", "TYPE")?
            .as_ref()
            .try_into()?;
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
        if force_platform_type.len() != used as usize {
            return Err(C3dParseError::InvalidParameterFormat(
                "FORCE_PLATFORM:TYPE".to_string(),
            ));
        }
        Ok(force_platform_type)
    }

    fn force_from_analog(&self, analog: [f32; 8]) -> [f32; 3] {
        match self {
            ForcePlatformType::Type3 => {
                return [
                    analog[0] + analog[1],
                    analog[2] + analog[3],
                    analog[4] + analog[5] + analog[6] + analog[7],
                ];
            }
            _ => {
                return [analog[0], analog[1], analog[2]];
            }
        }
    }

    fn center_of_pressure_from_analog(&self, analog: [f32; 8]) -> [f32; 2] {
        match self {
            ForcePlatformType::Type1 => [analog[3], analog[4]],
            ForcePlatformType::Type3 => {
                [(analog[0] + analog[1]) / 2., (analog[2] + analog[3]) / 2.]
            }
            _ => {
                let force_vector = [analog[0], analog[1], analog[2]];
                let moment_vector = [analog[3], analog[4], analog[5]];
                let center_of_pressure = cross_product(&force_vector, &moment_vector);
                [center_of_pressure[0], center_of_pressure[1]]
            }
        }
    }
}

fn cross_product(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ForcePlatformCorners {
    corners: [[f32; 3]; 4],
}

impl Deref for ForcePlatformCorners {
    type Target = [[f32; 3]; 4];

    fn deref(&self) -> &Self::Target {
        &self.corners
    }
}

impl DerefMut for ForcePlatformCorners {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.corners
    }
}

impl Index<usize> for ForcePlatformCorners {
    type Output = [f32; 3];

    fn index(&self, index: usize) -> &Self::Output {
        &self.corners[index]
    }
}

impl IndexMut<usize> for ForcePlatformCorners {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.corners[index]
    }
}

impl ForcePlatformCorners {
    pub fn new() -> Self {
        ForcePlatformCorners::default()
    }

    pub(crate) fn from_parameters(
        parameters: &mut Parameters,
        used: u16,
    ) -> Result<Vec<Self>, C3dParseError> {
        let mut corners = Vec::new();
        let parameter = parameters.remove_or_err("FORCE_PLATFORM", "CORNERS")?;
        match &parameter.data {
            ParameterData::Float(data) => {
                let dimensions: Vec<usize> =
                    parameter.dimensions.iter().map(|&x| x as usize).collect();
                if (dimensions.len() == 3 || dimensions.len() == 2)
                    && dimensions[1] == 4
                    && dimensions[0] == 3
                {
                    let num_plates = match dimensions.len() {
                        3 => dimensions[2],
                        2 => 1,
                        _ => Err(C3dParseError::InvalidData(
                            parameter.clone(),
                            "FORCE_PLATFORM_CORNERS".to_string(),
                        ))?,
                    };
                    for i in 0..num_plates {
                        let mut corner = [[0.0; 3]; 4];
                        for j in 0..4 {
                            let x = data[i * (dimensions[1] * dimensions[0]) + j * dimensions[0]];
                            let y =
                                data[i * (dimensions[1] * dimensions[0]) + j * dimensions[0] + 1];
                            let z =
                                data[i * (dimensions[1] * dimensions[0]) + j * dimensions[0] + 2];
                            corner[j] = [x, y, z];
                        }
                        corners.push(corner);
                    }
                } else {
                    return Err(C3dParseError::InvalidData(
                        parameter.clone(),
                        "FORCE_PLATFORM_CORNERS".to_string(),
                    ));
                }
            }
            _ => {
                return Err(C3dParseError::InvalidData(
                    parameter.clone(),
                    "FORCE_PLATFORM_CORNERS".to_string(),
                ));
            }
        }
        if corners.len() != used as usize {
            return Err(C3dParseError::InvalidData(
                parameter.clone(),
                "FORCE_PLATFORM_CORNERS".to_string(),
            ));
        }
        Ok(corners
            .iter()
            .map(|x| ForcePlatformCorners { corners: *x })
            .collect())
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ForcePlatformOrigin {
    origin: [f32; 3],
}

impl Deref for ForcePlatformOrigin {
    type Target = [f32; 3];

    fn deref(&self) -> &Self::Target {
        &self.origin
    }
}

impl DerefMut for ForcePlatformOrigin {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.origin
    }
}

impl ForcePlatformOrigin {
    pub fn new() -> Self {
        ForcePlatformOrigin::default()
    }

    pub(crate) fn from_parameters(
        parameters: &mut Parameters,
        used: u16,
    ) -> Result<Vec<Self>, C3dParseError> {
        let mut origin = Vec::new();
        let parameter = parameters.remove_or_err("FORCE_PLATFORM", "ORIGIN")?;
        match &parameter.data {
            ParameterData::Float(data) => {
                let dimensions: Vec<usize> =
                    parameter.dimensions.iter().map(|&x| x as usize).collect();
                if (dimensions.len() == 2 || dimensions.len() == 1) && dimensions[0] == 3 {
                    let num_plates = match dimensions.len() {
                        2 => dimensions[1],
                        1 => 1,
                        _ => unreachable!(),
                    };
                    for i in 0..num_plates {
                        let mut origin_point = [0.0; 3];
                        for j in 0..dimensions[0] {
                            origin_point[j] = data[i * dimensions[0] + j];
                        }
                        origin.push(origin_point);
                    }
                } else {
                    return Err(C3dParseError::InvalidData(
                        parameter.clone(),
                        "FORCE_PLATFORM_ORIGIN".to_string(),
                    ));
                }
            }
            _ => {
                return Err(C3dParseError::InvalidData(
                    parameter.clone(),
                    "FORCE_PLATFORM_ORIGIN".to_string(),
                ));
            }
        }
        if origin.len() != used as usize {
            return Err(C3dParseError::InvalidData(
                parameter.clone(),
                "FORCE_PLATFORM_ORIGIN".to_string(),
            ));
        }
        Ok(origin
            .iter()
            .map(|x| ForcePlatformOrigin {
                origin: [x[0], x[1], x[2]],
            })
            .collect())
    }
}

fn get_channels(
    parameters: &mut Parameters,
    used: u16,
    contains_type_3: bool,
) -> Result<Vec<[u8; 8]>, C3dParseError> {
    let channel_parameter = parameters.remove_or_err("FORCE_PLATFORM", "CHANNEL");
    let parameter = channel_parameter.unwrap();
    match &parameter.data {
        ParameterData::Integer(data) => {
            let dimensions: Vec<usize> = parameter.dimensions.iter().map(|&x| x as usize).collect();
            if dimensions.len() == 2 || dimensions.len() == 1 {
                if (dimensions.len() == 1 && used != 1)
                    || (dimensions.len() == 2 && dimensions[1] != used as usize)
                {
                    return Err(C3dParseError::InvalidData(
                        parameter.clone(),
                        "FORCE_PLATFORM_CHANNEL".to_string(),
                    ));
                }
                if dimensions[0] == 8 {
                    let mut array = Vec::new();
                    for i in 0..used as usize {
                        let mut channels = [0; 8];
                        for j in 0..8 {
                            channels[j] = data[i * 8 + j] as u8;
                        }
                        array.push(channels);
                    }
                    Ok(array)
                } else if dimensions[0] == 6 && !contains_type_3 {
                    let mut array = Vec::new();
                    for i in 0..used as usize {
                        let mut channels = [0; 8];
                        for j in 0..6 {
                            channels[j] = data[i * 6 + j] as u8;
                        }
                        array.push(channels);
                    }
                    Ok(array)
                } else {
                    Err(C3dParseError::InvalidData(
                        parameter.clone(),
                        "FORCE_PLATFORM_CHANNEL".to_string(),
                    ))
                }
            } else {
                Err(C3dParseError::InvalidData(
                    parameter.clone(),
                    "FORCE_PLATFORM_CHANNEL".to_string(),
                ))
            }
        }
        _ => Err(C3dParseError::InvalidData(
            parameter.clone(),
            "FORCE_PLATFORM_CHANNEL".to_string(),
        )),
    }
}

fn get_cal_matrix_vector(
    parameters: &mut Parameters,
    plate_type: &Vec<ForcePlatformType>,
) -> Result<Vec<Option<[[f32; 6]; 6]>>, C3dParseError> {
    let parameter = parameters.remove("FORCE_PLATFORM", "CAL_MATRIX");
    if parameter.is_none() {
        return Ok(vec![None; plate_type.len()]);
    }
    let num_type_4 = plate_type
        .iter()
        .filter(|x| **x == ForcePlatformType::Type4)
        .count();
    let parameter = parameter.unwrap();
    let cal_matrices = match &parameter.data {
        ParameterData::Float(data) => {
            let dimensions: Vec<usize> = parameter.dimensions.iter().map(|&x| x as usize).collect();
            if dimensions.len() == 3 || (dimensions.len() == 2 && data.len() == 36) {
                let mut array = Vec::new();
                if data.len() < 36 * num_type_4 {
                    return Err(C3dParseError::InvalidData(
                        parameter.clone(),
                        "FORCE_PLATFORM_CAL_MATRIX".to_string(),
                    ));
                }
                for platform in 0..data.len() / 36 {
                    let mut matrix = [[0.0; 6]; 6];
                    for i in 0..6 {
                        for j in 0..6 {
                            matrix[i][j] = data[platform * 36 + i * 6 + j];
                        }
                    }
                    array.push(matrix);
                }
                Some(array)
            } else {
                return Err(C3dParseError::InvalidData(
                    parameter.clone(),
                    "FORCE_PLATFORM_CAL_MATRIX".to_string(),
                ));
            }
        }
        _ => {
            return Err(C3dParseError::InvalidData(
                parameter.clone(),
                "FORCE_PLATFORM_CAL_MATRIX".to_string(),
            ))
        }
    };
    let cal_matrices = cal_matrices.unwrap();
    let mut cal_matrix_vec: Vec<Option<[[f32; 6]; 6]>> = vec![None; plate_type.len()];
    let mut count = 0;
    for i in 0..plate_type.len() {
        if plate_type[i] == ForcePlatformType::Type4 {
            cal_matrix_vec.push(Some(cal_matrices[count]));
            count += 1;
        }
    }
    Ok(cal_matrix_vec)
}
