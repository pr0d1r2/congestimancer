use super::CpuError;

#[test]
fn cpu_errors_render_for_the_user() {
    let missing = CpuError::Missing {
        file: "/proc/stat",
        key: "procs_running",
    };
    assert_eq!(missing.to_string(), "/proc/stat has no `procs_running`");
    let malformed = CpuError::malformed("/proc/loadavg", " x y \n");
    assert_eq!(malformed.to_string(), "malformed /proc/loadavg line: `x y`");
}
