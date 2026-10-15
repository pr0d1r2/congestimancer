use std::io;

use super::{nvml_error, nvml_gpus};

#[test]
fn an_absent_driver_or_feature_is_unsupported() {
    for status in [3, 9, 12] {
        let err = nvml_error("nvmlInit_v2", status);
        assert_eq!(err.kind(), io::ErrorKind::Unsupported, "{status}");
        assert_eq!(
            err.to_string(),
            format!("nvmlInit_v2: NVML status {status}")
        );
    }
}

#[test]
fn other_statuses_name_the_call() {
    let err = nvml_error("nvmlDeviceGetCount_v2", 999);
    assert_eq!(err.kind(), io::ErrorKind::Other);
    assert_eq!(err.to_string(), "nvmlDeviceGetCount_v2: NVML status 999");
}

#[cfg(target_os = "macos")]
#[test]
fn a_mac_has_no_nvml() {
    let err = nvml_gpus().err().map(|e| e.kind());
    assert_eq!(err, Some(io::ErrorKind::Unsupported));
}
