//! Switch parity (tests:V3, .:V18): every switch `docs/CONGESTION.md`
//! lists is parsed by the binary, and every gate switch reaches its gate.
//! The case table must name exactly the catalogue's switches, so a switch
//! added to the catalogue fails here until it has a case.

use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use super::{HERE, Outcome, cgmr};

/// The human catalogue (docs:V1).
const CATALOGUE: &str = include_str!("../../docs/CONGESTION.md");

/// What one switch's run must show.
#[derive(Clone, Copy)]
enum Reach {
    /// `check --json` reports a gate under this switch name.
    Gate(&'static str),
    /// `run` with no command accepts it: a selector, wait or launch switch.
    Run,
}

use Reach::{Gate, Run};

/// A network interface the net fixture has on Linux; macOS reads its
/// interfaces from the kernel, where the loopback always exists.
const IFACE: &str = if cfg!(target_os = "macos") {
    "lo0"
} else {
    "eth0"
};
/// The fixture tree with network interfaces, for `--net-iface`.
const NET: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/net/linux");

/// A switch, the argv that exercises it, and what it must reach.
type Case = (&'static str, &'static [&'static str], Reach);

/// Every catalogue switch, the argv that exercises it, and what it reaches.
/// `--host` is covered on its own: it needs a fake `ssh`.
const CASES: &[Case] = &[
    ("load", &["--load", "9"], Gate("load")),
    ("all-load", &["--all-load", "99"], Gate("all-load")),
    (
        "load-window",
        &["--load-window", "5m", "--load", "9"],
        Gate("load"),
    ),
    ("run-queue", &["--run-queue", "9"], Gate("run-queue")),
    (
        "all-run-queue",
        &["--all-run-queue", "99"],
        Gate("all-run-queue"),
    ),
    ("cpu-idle", &["--cpu-idle", "1"], Gate("cpu-idle")),
    (
        "cpu-pressure",
        &["--cpu-pressure", "50"],
        Gate("cpu-pressure"),
    ),
    ("steal", &["--steal", "50"], Gate("steal")),
    ("mem", &["--mem", "1KB"], Gate("mem")),
    ("all-mem", &["--all-mem", "1KB"], Gate("all-mem")),
    ("needs-mem", &["--needs-mem", "1KB"], Gate("needs-mem")),
    (
        "mem-pressure",
        &["--mem-pressure", "50"],
        Gate("mem-pressure"),
    ),
    (
        "mem-pressure-full",
        &["--mem-pressure-full", "50"],
        Gate("mem-pressure-full"),
    ),
    ("mem-level", &["--mem-level", "normal"], Gate("mem-level")),
    ("swap-rate", &["--swap-rate", "1MB/s"], Gate("swap-rate")),
    ("dirty", &["--dirty", "1GB"], Gate("dirty")),
    ("disk", &["--disk", "1KB"], Gate("disk")),
    ("inodes", &["--inodes", "1"], Gate("inodes")),
    (
        "disk-path",
        &["--disk-path", "/", "--disk", "1KB"],
        Gate("disk"),
    ),
    ("io-read", &["--io-read", "1GB/s"], Gate("io-read")),
    ("io-write", &["--io-write", "1GB/s"], Gate("io-write")),
    ("io", &["--io", "1GB/s"], Gate("io")),
    (
        "io-device",
        &["--io-device", "sda", "--io", "1GB/s"],
        Gate("io"),
    ),
    ("io-busy", &["--io-busy", "50"], Gate("io-busy")),
    ("io-latency", &["--io-latency", "100ms"], Gate("io-latency")),
    ("io-queue", &["--io-queue", "9"], Gate("io-queue")),
    ("io-pressure", &["--io-pressure", "50"], Gate("io-pressure")),
    (
        "io-pressure-full",
        &["--io-pressure-full", "50"],
        Gate("io-pressure-full"),
    ),
    ("net-rx", &["--net-rx", "1GB/s"], Gate("net-rx")),
    ("net-tx", &["--net-tx", "1GB/s"], Gate("net-tx")),
    ("net", &["--net", "1GB/s"], Gate("net")),
    (
        "net-iface",
        &["--net-iface", IFACE, "--net", "1GB/s"],
        Gate("net"),
    ),
    (
        "rtt",
        &["--rtt", "1s", "--rtt-host", "127.0.0.1"],
        Gate("rtt"),
    ),
    (
        "rtt-host",
        &["--rtt-host", "127.0.0.1", "--rtt", "1s"],
        Gate("rtt"),
    ),
    ("fds", &["--fds", "90"], Gate("fds")),
    ("pids", &["--pids", "90"], Gate("pids")),
    ("cool", &["--cool"], Gate("cool")),
    ("temp", &["--temp", "90C"], Gate("temp")),
    ("on-ac", &["--on-ac"], Gate("on-ac")),
    ("battery", &["--battery", "10"], Gate("battery")),
    ("no-low-power", &["--no-low-power"], Gate("no-low-power")),
    ("idle", &["--idle", "1s"], Gate("idle")),
    ("between", &["--between", "00:00-23:59"], Gate("between")),
    ("gpu", &["--gpu", "50"], Gate("gpu")),
    ("gpu-mem", &["--gpu-mem", "1KB"], Gate("gpu-mem")),
    (
        "gpu-device",
        &["--gpu-device", "0", "--gpu", "50"],
        Gate("gpu"),
    ),
    (
        "max-procs",
        &["--max-procs", "zz-none=1"],
        Gate("max-procs"),
    ),
    (
        "max-user-procs",
        &["--max-user-procs", "root=99999"],
        Gate("max-user-procs"),
    ),
    ("cgroup", &["--cgroup", "auto", "--load", "9"], Gate("load")),
    (
        "cpu-throttle",
        &["--cpu-throttle", "50"],
        Gate("cpu-throttle"),
    ),
    ("when", &["--when", "true"], Gate("when")),
    ("when-file", &["--when-file", HERE], Gate("when-file")),
    ("interval", &["--interval", "10ms"], Run),
    ("backoff", &["--backoff", "1s"], Run),
    ("timeout", &["--timeout", "1s"], Run),
    ("settle", &["--settle", "10ms"], Run),
    ("jitter", &["--jitter", "10ms"], Run),
    ("heartbeat", &["--heartbeat", "1s"], Run),
    ("slots", &["--slots", "2", "--slot-key", "parity"], Run),
    ("slot-key", &["--slot-key", "parity", "--slots", "2"], Run),
    ("fifo", &["--fifo", "--slot-key", "parity"], Run),
    (
        "alone-passes",
        &["--alone-passes", "--slots", "2", "--slot-key", "parity"],
        Run,
    ),
    ("rate", &["--rate", "99/1s", "--slot-key", "parity"], Run),
    (
        "spacing",
        &["--spacing", "10ms", "--slot-key", "parity"],
        Run,
    ),
    ("or-background", &["--or-background"], Run),
    ("verbose", &["--verbose"], Run),
    ("govern", &["--govern"], Run),
    ("background", &["--background"], Run),
];

/// Every `--switch` named in the catalogue's tables.
fn catalogue_switches() -> BTreeSet<&'static str> {
    let rows = CATALOGUE.lines().filter(|l| l.starts_with("| `--"));
    rows.flat_map(|row| row.split(['`', ' ', ',', '|']))
        .filter_map(|word| word.strip_prefix("--"))
        .filter(|name| !name.is_empty())
        .collect()
}

#[test]
fn the_cases_name_exactly_the_catalogue_switches() {
    let mut cased: BTreeSet<&str> = CASES.iter().map(|c| c.0).collect();
    cased.insert("host");
    assert_eq!(cased, catalogue_switches());
}

/// The argv for one case, its lock and runtime dirs in `state`.
fn command(
    name: &str,
    args: &[&str],
    reach: Reach,
    state: &std::path::Path,
) -> Result<assert_cmd::Command, Box<dyn Error>> {
    let mut cmd = cgmr()?;
    cmd.env("XDG_RUNTIME_DIR", state).env("TMPDIR", state);
    if name == "net-iface" {
        cmd.env("CGMR_FIXTURE_ROOT", NET);
    }
    match reach {
        Gate(_) => cmd.args(["check", "--json", "--interval", "10ms"]),
        Run => cmd.arg("run"),
    };
    cmd.args(args);
    Ok(cmd)
}

/// Whether a run that exited `code` printing `stdout` reached `reach`.
fn reached(reach: Reach, code: Option<i32>, stdout: &str) -> bool {
    match reach {
        Gate(gate) => {
            matches!(code, Some(0 | 1 | 69))
                && stdout.contains(&format!("\"switch\":\"{gate}\""))
        }
        Run => code == Some(0),
    }
}

/// Runs one case; an error names the switch and what went wrong.
fn reaches(name: &str, args: &[&str], reach: Reach) -> Outcome {
    let state = tempfile::tempdir()?;
    let out = command(name, args, reach, state.path())?.output()?;
    let code = out.status.code();
    let stdout = String::from_utf8_lossy(&out.stdout);
    if reached(reach, code, &stdout) {
        return Ok(());
    }
    let err = String::from_utf8_lossy(&out.stderr);
    Err(format!("--{name}: exit {code:?}\n{stdout}{err}").into())
}

#[test]
fn every_catalogue_switch_reaches_its_gate() -> Outcome {
    let failed: Vec<String> = CASES
        .iter()
        .filter_map(|(name, args, reach)| reaches(name, args, *reach).err())
        .map(|e| e.to_string())
        .collect();
    assert!(failed.is_empty(), "{}", failed.join("\n"));
    Ok(())
}

/// A fake `ssh` in `dir` that records its argv and answers like a remote
/// `cgmr check --json` whose gates pass.
fn fake_ssh(dir: &std::path::Path) -> Result<(), Box<dyn Error>> {
    let script = "#!/bin/sh\necho \"$@\" > \"$0.argv\"\n\
        echo '{\"schema_version\":1,\"host\":\"box\",\"signals\":[],\
        \"gates\":[{\"switch\":\"load\",\"threshold\":\"9\",\
        \"verdict\":\"pass\"}]}'\n";
    let ssh = dir.join("ssh");
    fs::write(&ssh, script)?;
    fs::set_permissions(&ssh, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

#[test]
fn host_sends_the_gates_over_ssh() -> Outcome {
    let bin = tempfile::tempdir()?;
    fake_ssh(bin.path())?;
    let path = std::env::var("PATH").unwrap_or_default();
    let path = format!("{}:{path}", bin.path().display());
    let args = ["check", "--host", "box", "--load", "9"];
    cgmr()?.env("PATH", path).args(args).assert().code(0);
    let argv = fs::read_to_string(bin.path().join("ssh.argv"))?;
    assert!(argv.contains("BatchMode=yes -- box"), "{argv}");
    assert!(argv.contains("'--load' '9'"), "{argv}");
    Ok(())
}
