use std::path::Path;

use super::{CgroupMode, Scope, parse_cgroup_mode};
use crate::cgroup::{Cgroup, CgroupError};
use crate::signal::Root;

/// The fixture tree `tests/fixtures/cgroup/<case>`, laid out like `/`.
fn root(case: &str) -> Root {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Root::new(dir.join("tests/fixtures/cgroup").join(case))
}

/// What `mode` resolves to in fixture `case`, as text.
fn resolve(case: &str, mode: &CgroupMode) -> Result<String, String> {
    Scope::resolve(&root(case), mode)
        .map(|scope| match scope {
            Scope::Cgroup(cgroup) => format!("cgroup {}", cgroup.path()),
            Scope::Host { reason } => format!("host: {reason}"),
        })
        .map_err(|reason| reason.as_str().to_owned())
}

#[test]
fn parses_the_three_modes() {
    let job = Cgroup::new("/user.slice/job");
    assert_eq!(parse_cgroup_mode("auto"), Ok(CgroupMode::Auto));
    assert_eq!(parse_cgroup_mode("host"), Ok(CgroupMode::Host));
    let path = parse_cgroup_mode("/user.slice/job");
    assert_eq!(path, Ok(CgroupMode::Path(job.clone())));
    let mounted = parse_cgroup_mode("/sys/fs/cgroup/user.slice/job/");
    assert_eq!(mounted, Ok(CgroupMode::Path(job)));
}

#[test]
fn a_relative_path_or_other_word_is_an_error() {
    let err = parse_cgroup_mode("user.slice").map_err(|e| e.to_string());
    assert_eq!(
        err,
        Err("--cgroup takes auto, host or an absolute cgroup path, \
             not `user.slice`"
            .to_owned())
    );
    let empty = parse_cgroup_mode("");
    assert!(matches!(empty, Err(CgroupError::Mode { .. })));
}

#[test]
fn auto_uses_a_cgroup_that_limits_an_ancestor_or_itself() {
    let found = resolve("limited", &CgroupMode::Auto);
    assert_eq!(found, Ok("cgroup /user.slice/job".to_owned()));
}

#[test]
fn auto_uses_the_cgroup_of_a_container() {
    let found = resolve("container", &CgroupMode::Auto);
    assert_eq!(found, Ok("cgroup /".to_owned()));
}

#[test]
fn auto_keeps_host_values_without_limits() {
    assert_eq!(
        resolve("unlimited", &CgroupMode::Auto),
        Ok("host: no cgroup limits this process".to_owned())
    );
}

#[test]
fn auto_keeps_host_values_without_cgroup_v2() {
    assert_eq!(
        resolve("v1", &CgroupMode::Auto),
        Ok("host: cgroup v1 is unsupported".to_owned())
    );
    assert_eq!(
        resolve("absent", &CgroupMode::Auto),
        Ok("host: /proc/self/cgroup: not present".to_owned())
    );
}

#[test]
fn host_mode_never_reads_the_cgroup() {
    assert_eq!(
        resolve("limited", &CgroupMode::Host),
        Ok("host: --cgroup host".to_owned())
    );
}

#[test]
fn a_given_path_is_used_even_without_limits() {
    let scope = Cgroup::new("/user.slice/session-1.scope");
    assert_eq!(
        resolve("unlimited", &CgroupMode::Path(scope)),
        Ok("cgroup /user.slice/session-1.scope".to_owned())
    );
}

#[test]
fn a_given_path_that_is_no_cgroup_is_unsupported() {
    let gone = CgroupMode::Path(Cgroup::new("/gone"));
    assert_eq!(
        resolve("limited", &gone),
        Err("/sys/fs/cgroup/gone is not a cgroup v2 directory".to_owned())
    );
}

#[test]
fn a_scope_hands_out_its_cgroup() {
    let job = Cgroup::new("/user.slice/job");
    assert_eq!(Scope::Cgroup(job.clone()).cgroup(), Some(&job));
    let host = Scope::Host {
        reason: "--cgroup host".into(),
    };
    assert_eq!(host.cgroup(), None);
}
