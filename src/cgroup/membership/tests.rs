use std::path::Path;

use super::{Cgroup, parse_proc_cgroup};
use crate::cgroup::CgroupError;
use crate::signal::Root;

/// The fixture tree `tests/fixtures/cgroup/<case>`, laid out like `/`.
fn root(case: &str) -> Root {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Root::new(dir.join("tests/fixtures/cgroup").join(case))
}

/// The text of `/proc/self/cgroup` in fixture `case`.
fn proc_self(case: &str) -> String {
    std::fs::read_to_string(root(case).join("/proc/self/cgroup"))
        .unwrap_or_default()
}

#[test]
fn a_v2_line_gives_the_path() {
    let path = parse_proc_cgroup(&proc_self("limited"));
    assert_eq!(path, Ok("/user.slice/job".to_owned()));
}

#[test]
fn the_namespace_root_is_a_path_too() {
    assert_eq!(parse_proc_cgroup("0::/\n"), Ok("/".to_owned()));
}

#[test]
fn a_deleted_cgroup_keeps_its_path() {
    let path = parse_proc_cgroup("0::/gone (deleted)\n");
    assert_eq!(path, Ok("/gone".to_owned()));
}

#[test]
fn v1_hierarchies_are_unsupported() {
    assert_eq!(parse_proc_cgroup(&proc_self("v1")), Err(CgroupError::V1));
}

#[test]
fn a_hybrid_host_counts_as_v1() {
    let parsed = parse_proc_cgroup(&proc_self("hybrid"));
    assert_eq!(parsed, Err(CgroupError::V1));
}

#[test]
fn malformed_text_is_an_error() {
    for text in ["", "0::\n", "0:/x\n", "x::/a\n", "0::/a\n0::/b\n"] {
        let parsed = parse_proc_cgroup(text);
        assert!(
            matches!(parsed, Err(CgroupError::Malformed { .. })),
            "{text:?} parsed as {parsed:?}"
        );
    }
}

#[test]
fn a_cgroup_names_its_files_under_the_mount() {
    let job = Cgroup::new("/user.slice/job");
    assert_eq!(job.path(), "/user.slice/job");
    assert_eq!(job.dir(), "/sys/fs/cgroup/user.slice/job");
    assert_eq!(job.file("cpu.max"), "/sys/fs/cgroup/user.slice/job/cpu.max");
    assert_eq!(Cgroup::new("/").file("cpu.max"), "/sys/fs/cgroup/cpu.max");
}

#[test]
fn a_cgroup_lists_itself_and_its_ancestors_below_the_root() {
    let chain = Cgroup::new("/user.slice/job").lineage();
    let paths: Vec<&str> = chain.iter().map(Cgroup::path).collect();
    assert_eq!(paths, ["/user.slice/job", "/user.slice"]);
    assert_eq!(Cgroup::new("/").lineage(), [Cgroup::new("/")]);
}

#[test]
fn detects_the_cgroup_of_this_process() {
    let found = Cgroup::detect(&root("limited"));
    assert_eq!(found, Ok(Cgroup::new("/user.slice/job")));
}

#[test]
fn detection_says_why_there_is_no_cgroup() {
    let why = |case: &str| {
        Cgroup::detect(&root(case)).map_err(|r| r.as_str().to_owned())
    };
    assert_eq!(why("v1"), Err("cgroup v1 is unsupported".to_owned()));
    assert_eq!(
        why("unmounted"),
        Err("cgroup2 is not mounted at /sys/fs/cgroup".to_owned())
    );
    assert_eq!(
        why("absent"),
        Err("/proc/self/cgroup: not present".to_owned())
    );
}
