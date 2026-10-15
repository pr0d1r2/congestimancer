use super::GpuError;
use crate::units::Count;

#[test]
fn malformed_names_the_file_and_the_trimmed_text() {
    let err = GpuError::malformed("gpu_busy_percent", " busy\n");
    assert_eq!(err.to_string(), "malformed gpu_busy_percent: `busy`");
}

#[test]
fn unknown_device_names_the_index_and_the_count() {
    let err = GpuError::UnknownDevice {
        index: Count(3),
        found: 2,
    };
    assert_eq!(
        err.to_string(),
        "--gpu-device 3: no such GPU (2 found, numbered from 0)"
    );
}
