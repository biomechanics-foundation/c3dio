use crate::{C3d, C3dWriteError};
use std::io::Write;
use std::path::PathBuf;
use grid::Grid;

#[derive(Debug, Clone)]
pub struct Sto {
    pub file_description: Option<String>,
    pub version: u8,
    pub in_degrees: bool,
    pub first_frame: usize,
    pub data_rate: f32,
    pub column_names: Vec<String>,
    pub data: Grid<f64>,
}

impl Sto {
    pub fn from_c3d(c3d: &C3d) -> Self {
        Sto {
            file_description: None,
            version: 1,
            in_degrees: false,
            first_frame: c3d.points.first_frame as usize,
            column_names: c3d.analog.labels.clone(),
            data_rate: c3d.analog.rate,
            data: c3d.analog.analog.clone(),
        }
    }

    pub fn write(&self, file_name: PathBuf) -> Result<(), C3dWriteError> {
        if file_name.is_dir() {
            return Err(C3dWriteError::InvalidFilePath(file_name));
        }
        if !file_name
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .to_lowercase()
            .eq("sto")
        {
            return Err(C3dWriteError::InvalidFileExtension(
                file_name.extension().unwrap().to_str().unwrap().to_string(),
            ));
        }
        let mut file = std::fs::File::create(file_name.clone()).map_err(|e| {
            C3dWriteError::WriteError(
                file_name.clone(),
                std::io::Error::new(std::io::ErrorKind::Other, e),
            )
        })?;
        let mut header = String::new();
        let description = match &self.file_description {
            None => file_name.to_str().unwrap().to_string(),
            Some(description) => description.clone(),
        };
        let degrees = if self.in_degrees { "yes" } else { "no" };
        header.push_str(&format!(
            "{}\n\
             version: {}\n\
             nRows: {}\n\
             nColumns: {}\n\
             inDegrees: {}\n\
             endheader\n",
            description,
            self.version,
            self.data.size().0,
            self.data.size().1,
            degrees,
        ));
        header.push_str("time\t");
        for i in 0..self.data.size().1 {
            if i >= self.column_names.len() {
                header.push_str(&format!("Column_{}\t", i));
            } else {
                header.push_str(&format!("{}\t", self.column_names[i]));
            }
        }
        header.push_str("\n");
        file.write_all(header.as_bytes()).map_err(|e| {
            C3dWriteError::WriteError(
                file_name.clone(),
                std::io::Error::new(std::io::ErrorKind::Other, e),
            )
        })?;
        for row in 0..self.data.size().0 {
            let mut row_string = String::new();
            row_string.push_str(&format!(
                "{}\t",
                (row + self.first_frame) as f32 / self.data_rate
            ));
            for column in 0..self.data.size().1 {
                row_string.push_str(&format!("{}\t", self.data[(row, column)]));
            }
            row_string.push_str("\n");
            file.write_all(row_string.as_bytes()).map_err(|e| {
                C3dWriteError::WriteError(
                    file_name.clone(),
                    std::io::Error::new(std::io::ErrorKind::Other, e),
                )
            })?;
        }
        Ok(())
    }
}
