use super::CgroupError;

#[test]
fn a_malformed_line_is_quoted() {
    let err = CgroupError::malformed("cpu.max", "1 2 3");
    assert_eq!(err.to_string(), "malformed cpu.max line: 1 2 3");
}

#[test]
fn a_missing_key_is_named() {
    let err = CgroupError::Missing {
        file: "cpu.stat",
        key: "throttled_usec",
    };
    assert_eq!(err.to_string(), "cpu.stat has no throttled_usec");
}

#[test]
fn v1_says_it_is_unsupported() {
    assert_eq!(CgroupError::V1.to_string(), "cgroup v1 is unsupported");
}
