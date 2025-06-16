//! TRC file format is a Motion Analysis Corporation file format for storing 3D marker data.
//! It is also used by OpenSim to store marker data.
use crate::C3d;
use crate::C3dWriteError;
use std::io::Write;
use std::path::PathBuf;

use crate::data::MarkerPoint;
use grid::Grid;

/// The TRC struct contains the data for writing a TRC file.
#[derive(Debug, Clone)]
pub struct Trc {
    pub path_file_type: u8,
    pub path_file_type_description: String,
    pub file_name: Option<PathBuf>,
    pub data_rate: f32,
    pub camera_rate: f32,
    pub num_frames: usize,
    pub units: [char; 4],
    pub marker_names: Vec<String>,
    pub first_frame: usize,
    pub data: Grid<MarkerPoint>,
}
impl Trc {
    pub fn from_c3d(c3d: &C3d) -> Self {
        let path_file_type = 4;
        let path_file_type_description = "(X/Y/Z)".to_string();
        let file_name = None;
        let data_rate = c3d.points.frame_rate;
        let camera_rate = c3d.points.frame_rate;
        let num_frames = c3d.points.size().0;
        let units = c3d.points.units.clone();
        let mut marker_names = c3d.points.labels.clone();
        if marker_names.len() > c3d.points.size().1 {
            marker_names = marker_names[0..c3d.points.size().1].to_vec();
        }
        let first_frame = c3d.points.first_frame as usize;
        let data = c3d.points.points.clone();
        Trc {
            path_file_type,
            path_file_type_description,
            file_name,
            data_rate,
            camera_rate,
            num_frames,
            units,
            marker_names,
            first_frame,
            data,
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
            .eq("trc")
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
        header.push_str(&format!(
            "PathFileType\t{}\t{}\t",
            self.path_file_type, self.path_file_type_description,
        ));
        header.push_str(file_name.to_string_lossy().to_string().as_str());
        header.push_str("\n");
        header.push_str("DataRate\tCameraRate\tNumFrames\tNumMarkers\tUnits\tOrigDataRate\tOrigDataStartFrame\tOrigNumFrames\n");
        header.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            self.data_rate,
            self.camera_rate,
            self.num_frames,
            self.marker_names.len(),
            self.units.iter().collect::<String>().trim(),
            self.data_rate,
            self.first_frame,
            self.num_frames,
        ));
        header.push_str("Frame#\tTime\t");
        for i in 0..self.data.size().1 {
            if i >= self.marker_names.len() {
                header.push_str(&format!("{}\t\t\t", i));
            } else {
                header.push_str(&format!("{}\t\t\t", self.marker_names[i].trim()));
            }
        }
        header.push_str("\n");
        header.push_str("\t\t");
        for i in 0..self.data.size().1 {
            header.push_str(&format!("X{}\tY{}\tZ{}\t", i + 1, i + 1, i + 1));
        }
        header.push_str("\n\n");
        file.write_all(header.as_bytes()).map_err(|e| {
            C3dWriteError::WriteError(
                file_name.clone(),
                std::io::Error::new(std::io::ErrorKind::Other, e),
            )
        })?;
        for i in 0..self.num_frames {
            let mut line = String::new();
            line.push_str(&format!(
                "{}\t{}\t",
                i + self.first_frame,
                (i + self.first_frame) as f32 / self.data_rate
            ));
            for j in 0..self.marker_names.len() {
                if self.data[(i, j)].residual < 0.0 {
                    line.push_str("\t\t\t");
                } else {
                    line.push_str(&format!(
                        "{}\t{}\t{}\t",
                        self.data[(i, j)][0],
                        self.data[(i, j)][1],
                        self.data[(i, j)][2]
                    ));
                }
            }
            line.push_str("\n");
            file.write_all(line.as_bytes()).map_err(|e| {
                C3dWriteError::WriteError(
                    file_name.clone(),
                    std::io::Error::new(std::io::ErrorKind::Other, e),
                )
            })?;
        }
        Ok(())
    }
}
