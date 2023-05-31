
#[cfg(test)]
mod tests {
    use crate::c3d::C3d;
    #[test]
    fn test_header_eq() {
        let header1 = C3d::header("res/Sample01/Eb015si.c3d").unwrap();
        let header2 = C3d::header("res/Sample01/Eb015pi.c3d").unwrap();
        let header3 = C3d::header("res/Sample01/Eb015vi.c3d").unwrap();
        let header4 = C3d::header("res/Sample01/Eb015sr.c3d").unwrap();
        let header5 = C3d::header("res/Sample01/Eb015pr.c3d").unwrap();
        let header6 = C3d::header("res/Sample01/Eb015vr.c3d").unwrap();
        assert!(&header1 == &header2);
        assert!(&header2 == &header3);
        assert!(&header3 == &header4);
        assert!(&header4 == &header5);
        assert!(&header5 == &header6);
    }

    #[test]
    fn test_parse_advanced_realtime_tracking() {
        // Advanced Realtime Tracking GmbH
        assert!(
            C3d::header("res/Sample00/Advanced Realtime Tracking GmbH/arthuman-sample.c3d").is_ok()
        );
        assert!(C3d::header(
            "res/Sample00/Advanced Realtime Tracking GmbH/arthuman-sample-fingers.c3d"
        )
        .is_ok());
    }

    #[test]
    fn test_parse_codamotion() {
        // Codamotion
        assert!(C3d::header("res/Sample00/Codamotion/codamotion_gaitwands_19970212.c3d").is_ok());
        assert!(C3d::header("res/Sample00/Codamotion/codamotion_gaitwands_20150204.c3d").is_ok());
    }

    #[test]
    fn test_parse_cometa() {
        // Cometa
        assert!(C3d::header("res/Sample00/Cometa Systems/EMG Data Cometa.c3d").is_ok());
    }

    #[test]
    fn test_parse_innovative_sports_training() {
        // Innovative Sports Training
        assert!(C3d::header("res/Sample00/Innovative Sports Training/Gait with EMG.c3d").is_ok());
        assert!(C3d::header("res/Sample00/Innovative Sports Training/Static Pose.c3d").is_ok());
    }

    #[test]
    fn test_parse_motion_analysis_corporation() {
        // Motion Analysis Corporation
        assert!(C3d::header("res/Sample00/Motion Analysis Corporation/Sample_Jump2.c3d").is_ok());
        assert!(C3d::header("res/Sample00/Motion Analysis Corporation/Walk1.c3d").is_ok());
    }

    #[test]
    fn test_parse_nexgen_ergonomics() {
        // NexGen Ergonomics
        assert!(C3d::header("res/Sample00/NexGen Ergonomics/test1.c3d").is_ok());
    }

    #[test]
    fn test_parse_vicon_motion_systems() {
        // Vicon Motion Systems
        assert!(C3d::header("res/Sample00/Vicon Motion Systems/TableTennis.c3d").is_ok());
        assert!(C3d::header(
            "res/Sample00/Vicon Motion Systems/pyCGM2 lower limb CGM24 Walking01.c3d"
        )
        .is_ok());
    }
}
