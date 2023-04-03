
use crate::parse::{C3dParseError, read_c3d, parse_basic_info, split_c3d};
use crate::processor::{ProcessorType, bytes_to_u16, bytes_to_f32};

#[derive(Debug)]
pub struct Header {
    parameter_start_block: u8,
    data_format: u8,
    processor_type: ProcessorType,
    num_3d_points_per_frame: u16,
    num_analog_samples_per_frame: u16,
    first_frame: u16,
    last_frame: u16,
    max_interpolation_gap: u16,
    scale_factor: f32,
    data_start_block: u16,
    analog_samples_per_3d_point: u16,
    frame_rate: f32,
    supports_event_labels: bool,
    num_time_events: u8,
    event_times: [f32; 18],
    event_display_flags: [bool; 18],
    event_labels: [[char; 4]; 18],
}

pub fn read_header_from_file(file: &str) -> Result<Header, C3dParseError> {
    let contents = read_c3d(file)?;

    let (parameter_start_block_index, data_start_block_index, processor_type) =
        parse_basic_info(&contents)?;

    let (header_blocks, _, _) = split_c3d(
        &contents,
        parameter_start_block_index,
        data_start_block_index,
    )?;

    parse_header(header_blocks, &processor_type)

}

pub fn parse_header(header: &[u8], processor_type: &ProcessorType) -> Result<Header, C3dParseError> {

    if header.len() < 512 {
        return Err(C3dParseError::InsufficientBlocks(format!("Header block is missing")));
    }

    let parameter_start_block = header[0];
    let data_format = header[1];

    let num_3d_points_per_frame = bytes_to_u16(&header[2..4], processor_type);
    let num_analog_samples_per_frame = bytes_to_u16(&header[4..6], processor_type);
    let first_frame = bytes_to_u16(&header[6..8], processor_type);
    let last_frame = bytes_to_u16(&header[8..10], processor_type);
    let max_interpolation_gap = bytes_to_u16(&header[10..12], processor_type);
    let scale_factor = bytes_to_f32(&header[12..16], processor_type);
    let data_start_block = bytes_to_u16(&header[16..18], processor_type);
    let analog_samples_per_3d_point = bytes_to_u16(&header[18..20], processor_type);
    let frame_rate = bytes_to_f32(&header[20..24], processor_type);
    let supports_event_labels_value = bytes_to_u16(&header[300..302], processor_type);
    let supports_event_labels = supports_event_labels_value == 0x3039;
    let num_time_events = header[302];
    let mut event_times = [0.0; 18];

    // event times start at byte 306
    for i in 0..18 {
        let start = 304 + (i * 4);
        let end = start + 4;
        event_times[i] = bytes_to_f32(&header[start..end], processor_type);
    }

    // event display flags start at byte 378
    let mut event_display_flags = [false; 18];

    for i in 0..18 {
        let index = 378 + i;
        let byte = header[index];
        if byte == 0x01 {
            event_display_flags[i] = true;
        }
    }

    // event labels start at byte 398
    let mut event_labels = [[0x00 as char; 4]; 18];

    for i in 0..18 {
        let start = 398 + (i * 4);
        let end = start + 4;

        let label_bytes = &header[start..end];

        for j in 0..4 {
            event_labels[i][j] = label_bytes[j] as char;
        }
    }

    let processor_type: ProcessorType = (*processor_type).clone();

    Ok(Header {
        parameter_start_block,
        data_format,
        processor_type,
        num_3d_points_per_frame,
        num_analog_samples_per_frame,
        first_frame,
        last_frame,
        max_interpolation_gap,
        scale_factor,
        data_start_block,
        analog_samples_per_3d_point,
        frame_rate,
        supports_event_labels,
        num_time_events,
        event_times,
        event_display_flags,
        event_labels,
    })
}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parameter Start Block: {}
            Data Format: {}
            Processor Type: {:?}
            Number of 3D Points per Frame: {}
            Number of Analog Samples per Frame: {}
            First Frame: {}
            Last Frame: {}
            Max Interpolation Gap: {}
            Scale Factor: {}
            Data Start Block: {}
            Analog Samples per 3D Point: {}
            Frame Rate: {}
            Supports Event Labels: {}
            Number of Time Events: {}
            Event Times: {}
            Event Display Flags: {}
            Event Labels: {}",
            self.parameter_start_block,
            self.data_format,
            self.processor_type,
            self.num_3d_points_per_frame,
            self.num_analog_samples_per_frame,
            self.first_frame,
            self.last_frame,
            self.max_interpolation_gap,
            self.scale_factor,
            self.data_start_block,
            self.analog_samples_per_3d_point,
            self.frame_rate,
            self.supports_event_labels,
            self.num_time_events,
            self.event_times
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            self.event_display_flags
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            self.event_labels
                .iter()
                .map(|x| match x {
                    ['\0', '\0', '\0', '\0'] => "".to_owned(),
                    _ => x.iter().collect::<String>(),
                })
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl std::cmp::PartialEq for Header {
    fn eq(&self, other: &Self) -> bool {
        self.parameter_start_block == other.parameter_start_block
            && self.data_format == other.data_format
            && self.num_3d_points_per_frame == other.num_3d_points_per_frame
            && self.num_analog_samples_per_frame == other.num_analog_samples_per_frame
            && self.first_frame == other.first_frame
            && self.last_frame == other.last_frame
            && self.max_interpolation_gap == other.max_interpolation_gap
            && self.data_start_block == other.data_start_block
            && self.analog_samples_per_3d_point == other.analog_samples_per_3d_point
            && self.frame_rate == other.frame_rate
            && self.supports_event_labels == other.supports_event_labels
            && self.num_time_events == other.num_time_events
            && self.event_times == other.event_times
            && self.event_display_flags == other.event_display_flags
            && self.event_labels == other.event_labels
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_header_eq() {
        let header1 = read_header_from_file("res/Sample01/Eb015si.c3d").unwrap();
        let header2 = read_header_from_file("res/Sample01/Eb015pi.c3d").unwrap();
        let header3 = read_header_from_file("res/Sample01/Eb015vi.c3d").unwrap();
        let header4 = read_header_from_file("res/Sample01/Eb015sr.c3d").unwrap();
        let header5 = read_header_from_file("res/Sample01/Eb015pr.c3d").unwrap();
        let header6 = read_header_from_file("res/Sample01/Eb015vr.c3d").unwrap();
        assert!(&header1 == &header2);
        assert!(&header2 == &header3);
        assert!(&header3 == &header4);
        assert!(&header4 == &header5);
        assert!(&header5 == &header6);
    }

    #[test]
    fn test_parse_advanced_realtime_tracking() {
        // Advanced Realtime Tracking GmbH
        assert!(read_header_from_file(
            "res/Sample00/Advanced Realtime Tracking GmbH/arthuman-sample.c3d"
        )
        .is_ok());
        assert!(read_header_from_file(
            "res/Sample00/Advanced Realtime Tracking GmbH/arthuman-sample-fingers.c3d"
        )
        .is_ok());
    }

    #[test]
    fn test_parse_codamotion() {
        // Codamotion
        assert!(read_header_from_file(
            "res/Sample00/Codamotion/codamotion_gaitwands_19970212.c3d"
        )
        .is_ok());
        assert!(read_header_from_file(
            "res/Sample00/Codamotion/codamotion_gaitwands_20150204.c3d"
        )
        .is_ok());
    }

    #[test]
    fn test_parse_cometa() {
        // Cometa
        assert!(read_header_from_file("res/Sample00/Cometa Systems/EMG Data Cometa.c3d").is_ok());
    }

    #[test]
    fn test_parse_innovative_sports_training() {
        // Innovative Sports Training
        assert!(read_header_from_file(
            "res/Sample00/Innovative Sports Training/Gait with EMG.c3d"
        )
        .is_ok());
        assert!(
            read_header_from_file("res/Sample00/Innovative Sports Training/Static Pose.c3d")
                .is_ok()
        );
    }

    #[test]
    fn test_parse_motion_analysis_corporation() {
        // Motion Analysis Corporation
        assert!(read_header_from_file(
            "res/Sample00/Motion Analysis Corporation/Sample_Jump2.c3d"
        )
        .is_ok());
        assert!(
            read_header_from_file("res/Sample00/Motion Analysis Corporation/Walk1.c3d").is_ok()
        );
    }

    #[test]
    fn test_parse_nexgen_ergonomics() {
        // NexGen Ergonomics
        assert!(read_header_from_file("res/Sample00/NexGen Ergonomics/test1.c3d").is_ok());
    }

    #[test]
    fn test_parse_vicon_motion_systems() {
        // Vicon Motion Systems
        assert!(
            read_header_from_file("res/Sample00/Vicon Motion Systems/TableTennis.c3d").is_ok()
        );
        assert!(read_header_from_file(
            "res/Sample00/Vicon Motion Systems/pyCGM2 lower limb CGM24 Walking01.c3d"
        )
        .is_ok());
    }
}
