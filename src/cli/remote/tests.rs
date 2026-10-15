use std::time::{Duration, Instant};

use super::{Remote, ScriptedSsh, check_on, gate_args, wait_remote};
use crate::cli::entry::ran;
use crate::cli::layers::{Layer, Resolved};
use crate::gate::Overall;
use crate::launch::{Judge, Slots};
use crate::remote::{SshReply, parse_host};
use crate::signal::ManualClock;
use crate::wait::{Outcome, Pacing};

/// A remote `check --json` whose one gate said `verdict`.
fn reply(verdict: &str) -> String {
    format!(
        "{{\"schema_version\":1,\"host\":\"box\",\"signals\":[],\
         \"gates\":[{{\"switch\":\"load\",\"threshold\":\"2\",\
         \"verdict\":\"{verdict}\"}}]}}"
    )
}

/// One round on `box` through `ssh`.
fn round(ssh: &mut ScriptedSsh) -> Option<Overall> {
    let mut log = Vec::new();
    let host = parse_host("box").ok()?;
    let mut remote = remote(ssh, host, &mut log);
    remote.round().ok()
}

fn remote<'a>(
    ssh: &'a mut ScriptedSsh,
    host: crate::remote::Host,
    log: &'a mut Vec<u8>,
) -> Remote<'a> {
    Remote {
        ssh,
        host,
        args: vec!["--load".to_owned(), "2".to_owned()],
        log,
        verbose: true,
    }
}

/// The switches a run on a host sets, one of each kind.
fn switches() -> Resolved {
    let set = [
        ("load", &["2"][..]),
        ("cool", &["true"]),
        ("on-ac", &["false"]),
        ("disk-path", &["/var"]),
        ("timeout", &["1m"]),
        ("max-procs", &["a=1", "b=2"]),
    ];
    set.iter().fold(Resolved::default(), |r, (name, values)| {
        r.with(name, values, Layer::Flag)
    })
}

#[test]
fn gate_args_carry_gates_and_selectors_only() {
    let want = [
        "--load",
        "2",
        "--disk-path",
        "/var",
        "--cool",
        "--max-procs",
        "a=1",
        "--max-procs",
        "b=2",
    ];
    assert_eq!(gate_args(&switches()), want);
}

#[test]
fn a_round_is_what_the_host_says() {
    assert_eq!(
        round(&mut ScriptedSsh::always(0, &reply("pass"))),
        Some(Overall::Pass)
    );
    assert_eq!(
        round(&mut ScriptedSsh::always(1, &reply("fail"))),
        Some(Overall::Fail)
    );
    let unsupported = ScriptedSsh::always(1, &reply("unsupported"));
    assert_eq!(round(&mut { unsupported }), Some(Overall::Unsupported));
}

#[test]
fn an_unreachable_host_fails_the_round() {
    assert_eq!(
        round(&mut ScriptedSsh::always(255, "")),
        Some(Overall::Fail)
    );
    assert_eq!(round(&mut ScriptedSsh::always(127, "")), None);
}

#[test]
fn the_judge_pauses_on_an_error() {
    let mut ssh = ScriptedSsh::always(127, "");
    let mut log = Vec::new();
    let host = parse_host("box");
    let judged =
        host.map(|h| remote(&mut ssh, h, &mut log).judge(Instant::now()));
    assert_eq!(judged.ok(), Some(Overall::Fail));
}

#[test]
fn check_on_maps_the_round_to_an_exit() {
    let mut ssh = ScriptedSsh::always(1, &reply("fail"));
    let mut log = Vec::new();
    let host = parse_host("box");
    let code = host.map(|h| check_on(&mut remote(&mut ssh, h, &mut log)));
    assert_eq!(code.ok().and_then(Result::ok), Some(1));
}

#[test]
fn wait_remote_releases_once_the_host_passes() {
    let replies =
        ["fail", "pass"].map(|v| SshReply::new(Some(0), reply(v), ""));
    let mut ssh = ScriptedSsh {
        replies: replies.to_vec(),
        calls: Vec::new(),
    };
    let mut log = Vec::new();
    let Ok(host) = parse_host("box") else { return };
    let mut remote = remote(&mut ssh, host, &mut log);
    let pacing = Pacing::new(Duration::from_secs(1));
    let mut clock = ManualClock::new();
    let outcome =
        wait_remote(&mut remote, &pacing, &mut clock, &mut Slots::none());
    assert!(matches!(outcome, Ok(Outcome::Released(None))));
    assert_eq!(clock.sleeps(), [Duration::from_secs(1)]);
}

#[test]
fn wait_remote_times_out() {
    let mut ssh = ScriptedSsh::always(1, &reply("fail"));
    let mut log = Vec::new();
    let Ok(host) = parse_host("box") else { return };
    let mut remote = remote(&mut ssh, host, &mut log);
    let mut pacing = Pacing::new(Duration::from_secs(1));
    pacing.timeout = Some(Duration::from_secs(2));
    let mut clock = ManualClock::new();
    let outcome =
        wait_remote(&mut remote, &pacing, &mut clock, &mut Slots::none());
    assert!(matches!(outcome, Ok(Outcome::TimedOut(None))));
}

#[test]
fn check_and_run_go_to_the_host() {
    let mut ssh = ScriptedSsh::always(0, &reply("pass"));
    let checked =
        ran(&["check", "--host", "box", "--load", "2"], &[], &mut ssh);
    assert_eq!(checked.code, 0);
    let words = ssh.calls.first().cloned().unwrap_or_default();
    assert!(
        words.ends_with(&["'--load'".to_owned(), "'2'".to_owned()]),
        "{words:?}"
    );
    let run = ran(&["--host", "box", "--load", "2"], &[], &mut ssh);
    assert_eq!(run.code, 0);
}

#[test]
fn run_on_a_host_without_cgmr_exits_69() {
    let mut ssh = ScriptedSsh::always(127, "");
    let run = ran(&["--host", "box", "--load", "2"], &[], &mut ssh);
    assert_eq!(run.code, 69);
    let unsupported = ScriptedSsh::always(1, &reply("unsupported"));
    let run = ran(&["--host", "box"], &[], &mut { unsupported });
    assert_eq!(run.code, 69);
}
