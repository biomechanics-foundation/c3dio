//! Structure for storing SEG parameters.
use crate::parameters::{Parameter, Parameters};
use crate::processor::Processor;
use crate::{C3dParseError, C3dWriteError};
use grid::Grid;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Seg {
    pub marker_diameter: Option<f32>,
    pub data_limits: Option<Grid<f32>>,
    pub acc_factor: Option<f32>,
    pub noise_factor: Option<f32>,
    pub residual_error_factor: Option<f32>,
    pub intersection_limit: Option<f32>,
}

impl PartialEq for Seg {
    fn eq(&self, other: &Self) -> bool {
        let data_limits_eq = if let Some(data_limits) = &self.data_limits {
            if let Some(other_data_limits) = &other.data_limits {
                data_limits.flatten() == other_data_limits.flatten()
            } else {
                false
            }
        } else if other.data_limits.is_some() {
            false
        } else {
            true
        };
        self.marker_diameter == other.marker_diameter
            && data_limits_eq
            && self.acc_factor == other.acc_factor
            && self.noise_factor == other.noise_factor
            && self.residual_error_factor == other.residual_error_factor
            && self.intersection_limit == other.intersection_limit
    }
}

impl ToString for Seg {
    fn to_string(&self) -> String {
        let mut string = String::new();
        if let Some(marker_diameter) = &self.marker_diameter {
            string.push_str(&format!("Marker Diameter: {}\n", marker_diameter));
        }
        if let Some(data_limits) = &self.data_limits {
            string.push_str(&format!("Data Limits: {:?}\n", data_limits));
        }
        if let Some(acc_factor) = &self.acc_factor {
            string.push_str(&format!("Acc Factor: {}\n", acc_factor));
        }
        if let Some(noise_factor) = &self.noise_factor {
            string.push_str(&format!("Noise Factor: {}\n", noise_factor));
        }
        if let Some(residual_error_factor) = &self.residual_error_factor {
            string.push_str(&format!(
                "Residual Error Factor: {}\n",
                residual_error_factor
            ));
        }
        if let Some(intersection_limit) = &self.intersection_limit {
            string.push_str(&format!("Intersection Limit: {}\n", intersection_limit));
        }
        string
    }
}

impl Seg {
    pub(crate) fn from_parameters(parameters: &mut Parameters) -> Result<Self, C3dParseError> {
        let marker_diameter_parameter = parameters.remove("SEG", "MARKER_DIAMETER");
        let marker_diameter: Option<f32> = match marker_diameter_parameter {
            None => None,
            Some(parameter) => Some(parameter.as_ref().try_into()?),
        };
        let data_limits_parameter = parameters.remove("SEG", "DATA_LIMITS");
        let data_limits: Option<Grid<f32>> = match data_limits_parameter {
            None => None,
            Some(parameter) => Some(parameter.as_ref().try_into()?),
        };
        let acc_factor = parameters.remove("SEG", "ACC_FACTOR");
        let acc_factor: Option<f32> = match acc_factor {
            None => None,
            Some(parameter) => Some(parameter.as_ref().try_into()?),
        };
        let noise_factor = parameters.remove("SEG", "NOISE_FACTOR");
        let noise_factor: Option<f32> = match noise_factor {
            None => None,
            Some(parameter) => Some(parameter.as_ref().try_into()?),
        };
        let residual_error_factor = parameters.remove("SEG", "RESIDUAL_ERROR_FACTOR");
        let residual_error_factor: Option<f32> = match residual_error_factor {
            None => None,
            Some(parameter) => Some(parameter.as_ref().try_into()?),
        };
        let intersection_limit = parameters.remove("SEG", "INTERSECTION_LIMIT");
        let intersection_limit: Option<f32> = match intersection_limit {
            None => None,
            Some(parameter) => Some(parameter.as_ref().try_into()?),
        };
        Ok(Seg {
            marker_diameter,
            data_limits,
            acc_factor,
            noise_factor,
            residual_error_factor,
            intersection_limit,
        })
    }

    pub(crate) fn write(
        &self,
        processor: &Processor,
        group_names_to_ids: &HashMap<String, usize>,
    ) -> Result<Vec<u8>, C3dWriteError> {
        let mut bytes = Vec::new();
        if self.marker_diameter.is_some() {
            bytes.extend(Parameter::float(self.marker_diameter.unwrap()).write(
                processor,
                "MARKER_DIAMETER".to_string(),
                group_names_to_ids["SEG"],
                false,
            )?);
        }
        if self.data_limits.is_some() {
            bytes.extend(Parameter::float_grid(self.data_limits.clone().unwrap()).write(
                processor,
                "DATA_LIMITS".to_string(),
                group_names_to_ids["SEG"],
                false,
            )?);
        }
        if self.acc_factor.is_some() {
            bytes.extend(Parameter::float(self.acc_factor.unwrap()).write(
                processor,
                "ACC_FACTOR".to_string(),
                group_names_to_ids["SEG"],
                false,
            )?);
        }
        if self.noise_factor.is_some() {
            bytes.extend(Parameter::float(self.noise_factor.unwrap()).write(
                processor,
                "NOISE_FACTOR".to_string(),
                group_names_to_ids["SEG"],
                false,
            )?);
        }
        if self.residual_error_factor.is_some() {
            bytes.extend(Parameter::float(self.residual_error_factor.unwrap()).write(
                processor,
                "RESIDUAL_ERROR_FACTOR".to_string(),
                group_names_to_ids["SEG"],
                false,
            )?);
        }
        if self.intersection_limit.is_some() {
            bytes.extend(Parameter::float(self.intersection_limit.unwrap()).write(
                processor,
                "INTERSECTION_LIMIT".to_string(),
                group_names_to_ids["SEG"],
                false,
            )?);
        }
        Ok(bytes)
    }
}
