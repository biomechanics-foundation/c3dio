use c3dio::c3d::{test_load_file, ProcessStep};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_all_sample_files() {
    let top_dir = "tests/c3d.org-sample-files";
    let ignored_paths = vec![
        "tests/c3d.org-sample-files/Sample06/MACsample.c3d",
        "tests/c3d.org-sample-files/Sample11/evart.c3d",
        "tests/c3d.org-sample-files/Sample13/golfswing.c3d",
        "tests/c3d.org-sample-files/Sample13/golfswing1.c3d",
        "tests/c3d.org-sample-files/Sample13/Dance.c3d",
        "tests/c3d.org-sample-files/Sample13/Dance1.c3d",
        "tests/c3d.org-sample-files/Sample18/bad_parameter_section.c3d",
        "tests/c3d.org-sample-files/Sample24/MotionMonitorC3D.c3d",
    ];
    let mut files = Vec::new();
    let mut dirs: Vec<PathBuf> = Vec::new();
    dirs.push(PathBuf::from(top_dir));
    while dirs.len() > 0 {
        let dir = dirs.pop().unwrap();
        let paths = fs::read_dir(dir).unwrap();
        for path in paths {
            // check if directory
            let path = path.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
                continue;
            }
            let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
            if file_name.ends_with(".c3d") {
                files.push(path.to_str().unwrap().to_string());
            }
        }
    }
    let mut file_tests = HashMap::new();
    for file_name in files {
        if file_name.ends_with(".c3d") && !ignored_paths.contains(&file_name.as_str()) {
            file_tests.insert(file_name.clone(), test_load_file(&file_name));
        }
    }
    let mut parsed_all_files = true;
    for (file_name, step) in file_tests {
        if step != ProcessStep::Complete {
            println!("{}: {:?}", file_name, step);
            parsed_all_files = false;
        }
    }
    assert!(parsed_all_files);
}
