use super::MemError;

#[test]
fn mem_errors_render_for_the_user() {
    let missing = MemError::Missing {
        file: "/proc/vmstat",
        key: "pswpin",
    };
    assert_eq!(missing.to_string(), "/proc/vmstat has no `pswpin`");
    let malformed = MemError::malformed("/proc/vmstat", " pswpin x \n");
    assert_eq!(
        malformed.to_string(),
        "malformed /proc/vmstat line: `pswpin x`"
    );
}
