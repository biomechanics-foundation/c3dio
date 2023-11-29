use crate::parameters::{Parameter, ParameterData, Parameters};
use crate::processor::Processor;
use crate::{C3dIoError, C3dParseError};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ManufacturerVersion {
    String(String),
    Float(f32),
    Array(Vec<i16>),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Manufacturer {
    pub company: Option<String>,
    pub software: Option<String>,
    pub version: Option<ManufacturerVersion>,
    pub edited: Option<Vec<String>>,
}

impl ToString for Manufacturer {
    fn to_string(&self) -> String {
        let mut string = String::new();
        if let Some(company) = &self.company {
            string.push_str(&format!("Company: {}\n", company));
        }
        if let Some(software) = &self.software {
            string.push_str(&format!("Software: {}\n", software));
        }
        if let Some(version) = &self.version {
            match version {
                ManufacturerVersion::String(version) => {
                    string.push_str(&format!("Version: {}\n", version));
                }
                ManufacturerVersion::Float(version) => {
                    string.push_str(&format!("Version: {}\n", version));
                }
                ManufacturerVersion::Array(version) => {
                    string.push_str(&format!("Version: {:?}\n", version));
                }
            }
        }
        if let Some(edited) = &self.edited {
            string.push_str(&format!("Edited: {:?}\n", edited));
        }
        string
    }
}

impl Manufacturer {
    pub fn new() -> Self {
        Manufacturer::default()
    }

    pub(crate) fn from_parameters(parameters: &mut Parameters) -> Result<Self, C3dParseError> {
        let company = parameters.remove("MANUFACTURER", "COMPANY");
        let company: Option<String> = match company {
            None => None,
            Some(parameter) => Some(parameter.as_ref().try_into()?),
        };
        let software = parameters.remove("MANUFACTURER", "SOFTWARE");
        let software: Option<String> = match software {
            None => None,
            Some(parameter) => Some(parameter.as_ref().try_into()?),
        };
        let version = get_manufacturer_version(parameters);
        let edited = parameters.remove("MANUFACTURER", "EDITED");
        let edited: Option<Vec<String>> = match edited {
            None => None,
            Some(parameter) => Some(parameter.as_ref().try_into()?),
        };
        Ok(Manufacturer {
            company,
            software,
            version,
            edited,
        })
    }

    pub(crate) fn write(
        &self,
        processor: &Processor,
        group_names_to_ids: &HashMap<String, usize>,
    ) -> Result<Vec<u8>, C3dIoError> {
        let mut bytes = Vec::new();
        if let Some(company) = &self.company {
            bytes.extend(Parameter::chars(company.chars().collect())?.write(
                processor,
                "COMPANY".to_string(),
                group_names_to_ids["MANUFACTURER"],
                false,
            )?);
        }
        if let Some(software) = &self.software {
            bytes.extend(Parameter::chars(software.chars().collect())?.write(
                processor,
                "SOFTWARE".to_string(),
                group_names_to_ids["MANUFACTURER"],
                false,
            )?);
        }
        if let Some(version) = &self.version {
            match version {
                ManufacturerVersion::String(version) => {
                    bytes.extend(Parameter::chars(version.chars().collect())?.write(
                        processor,
                        "VERSION".to_string(),
                        group_names_to_ids["MANUFACTURER"],
                        false,
                    )?);
                }
                ManufacturerVersion::Float(version) => {
                    bytes.extend(Parameter::float(*version).write(
                        processor,
                        "VERSION".to_string(),
                        group_names_to_ids["MANUFACTURER"],
                        false,
                    )?);
                }
                ManufacturerVersion::Array(version) => {
                    if version.len() != 0 {
                        bytes.extend(Parameter::integers(version.clone())?.write(
                            processor,
                            "VERSION".to_string(),
                            group_names_to_ids["MANUFACTURER"],
                            false,
                        )?);
                    }
                }
            }
        }
        if let Some(edited) = &self.edited {
            bytes.extend(Parameter::strings(edited.clone()).write(
                processor,
                "EDITED".to_string(),
                group_names_to_ids["MANUFACTURER"],
                false,
            )?);
        }
        Ok(bytes)
    }
}

fn get_manufacturer_version(parameters: &mut Parameters) -> Option<ManufacturerVersion> {
    let version = parameters.remove("MANUFACTURER", "VERSION");
    if version.is_none() {
        None
    } else {
        let version = version.unwrap();
        match version.data {
            ParameterData::Char(_) => {
                let version: Result<String, C3dParseError> = version.as_ref().try_into();
                let version = match version {
                    Ok(version) => version,
                    Err(_) => return None,
                };
                Some(ManufacturerVersion::String(version))
            }
            ParameterData::Float(_) => {
                let version: Result<f32, C3dParseError> = version.as_ref().try_into();
                let version = match version {
                    Ok(version) => version,
                    Err(_) => return None,
                };
                Some(ManufacturerVersion::Float(version))
            }
            ParameterData::Integer(_) => {
                let version: Result<Vec<i16>, C3dParseError> = version.as_ref().try_into();
                let version = match version {
                    Ok(version) => version,
                    Err(_) => return None,
                };
                Some(ManufacturerVersion::Array(version))
            }
            _ => None,
        }
    }
}
