use std::ffi::OsString;
use std::io;
use std::path::Path;

use super::{World, fixture_root, wrote};
use crate::cli::env::Env;
use crate::cli::exit::FAIL;
use crate::cli::remote::ScriptedSsh;
use crate::report::Style;
use crate::signal::Root;

fn env(vars: &[(&str, &str)]) -> Env {
    let os = |s: &&str| OsString::from(s);
    Env::new(vars.iter().map(|(k, v)| (os(k), os(v))))
}

/// The hostname a world over `root` and `vars` reports.
fn hostname(root: Root, vars: &[(&str, &str)]) -> String {
    let (mut out, mut err, mut ssh) =
        (Vec::new(), Vec::new(), ScriptedSsh::default());
    let mut world = World::new(env(vars), &mut out, &mut err, &mut ssh);
    world.root = root;
    world.hostname()
}

#[test]
fn the_fixture_root_comes_from_the_environment() {
    let fixture = env(&[("CGMR_FIXTURE_ROOT", "tests/fixtures/cpu/linux")]);
    let root = fixture_root(&fixture);
    assert_eq!(root.path(), Path::new("tests/fixtures/cpu/linux"));
    assert_eq!(fixture_root(&Env::default()), Root::system());
    assert_eq!(
        fixture_root(&env(&[("CGMR_FIXTURE_ROOT", "")])),
        Root::system()
    );
}

#[test]
fn the_hostname_falls_back_to_the_environment() {
    let empty = Root::new("no/such/root");
    assert_eq!(hostname(empty.clone(), &[("HOSTNAME", "box")]), "box");
    assert_eq!(hostname(empty, &[]), "localhost");
}

#[test]
fn a_closed_pipe_is_no_failure() {
    assert!(wrote(Err(io::ErrorKind::BrokenPipe.into())).is_ok());
    let failed = wrote(Err(io::ErrorKind::Other.into()));
    assert_eq!(failed.err().map(|f| f.code), Some(FAIL));
}

#[test]
fn no_color_turns_colour_off() {
    let (mut out, mut err, mut ssh) =
        (Vec::new(), Vec::new(), ScriptedSsh::default());
    let world =
        World::new(env(&[("NO_COLOR", "1")]), &mut out, &mut err, &mut ssh);
    assert_eq!(world.style(true), Style::Plain);
}

#[test]
fn the_hostname_comes_from_the_kernel_under_the_root() {
    let fixture = Root::new("tests/fixtures/cli");
    assert_eq!(hostname(fixture, &[("HOSTNAME", "box")]), "fixture-box");
}

#[test]
fn terminals_are_detected() {
    let tty = super::Tty::detect();
    assert_eq!(
        tty,
        super::Tty {
            out: tty.out,
            err: tty.err
        }
    );
}
