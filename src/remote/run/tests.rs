use std::io;

use super::{Round, SshReply, SystemSsh, Transport, check_remote};
use crate::gate::Overall;
use crate::remote::{Host, RemoteError, SSH, parse_host, ssh_argv};

const V1: &str = include_str!("../../../tests/fixtures/remote/check-v1.json");
const V2: &str = include_str!("../../../tests/fixtures/remote/check-v2.json");

/// A transport that answers with a canned reply and keeps what it ran.
struct Fake {
    reply: io::Result<SshReply>,
    ran: Vec<(String, Vec<String>)>,
}

impl Fake {
    fn answering(reply: SshReply) -> Self {
        Self {
            reply: Ok(reply),
            ran: Vec::new(),
        }
    }
}

impl Transport for Fake {
    fn run(&mut self, program: &str, argv: &[String]) -> io::Result<SshReply> {
        self.ran.push((program.to_owned(), argv.to_vec()));
        match &self.reply {
            Ok(reply) => Ok(reply.clone()),
            Err(err) => Err(io::Error::new(err.kind(), err.to_string())),
        }
    }
}

fn host() -> Host {
    parse_host("box").unwrap_or_else(|err| unreachable!("{err}"))
}

fn gates() -> Vec<String> {
    vec!["--load".to_owned(), "2".to_owned()]
}

fn round(fake: &mut Fake) -> Result<Round, RemoteError> {
    check_remote(fake, &host(), &gates())
}

#[test]
fn it_runs_ssh_with_the_built_argv() {
    let mut fake = Fake::answering(SshReply::new(Some(1), V1, ""));
    let _ = round(&mut fake);
    assert_eq!(
        fake.ran,
        vec![(SSH.to_owned(), ssh_argv(&host(), &gates()))]
    );
}

#[test]
fn a_failing_check_is_a_report() {
    let mut fake = Fake::answering(SshReply::new(Some(1), V1, ""));
    let Ok(Round::Report(report)) = round(&mut fake) else {
        unreachable!("not a report");
    };
    assert_eq!(report.host, "box");
    assert_eq!(report.overall(), Overall::Unsupported);
}

#[test]
fn a_passing_check_is_a_report() {
    let pass =
        "{\"schema_version\":1,\"host\":\"b\",\"signals\":[],\"gates\":[]}";
    let mut fake = Fake::answering(SshReply::new(Some(0), pass, ""));
    let overall = match round(&mut fake) {
        Ok(Round::Report(report)) => Some(report.overall()),
        _ => None,
    };
    assert_eq!(overall, Some(Overall::Pass));
}

#[test]
fn another_schema_is_exit_69() {
    let mut fake = Fake::answering(SshReply::new(Some(0), V2, ""));
    assert_eq!(round(&mut fake).map_err(|e| e.exit_code()), Err(69));
}

#[test]
fn the_system_transport_runs_a_program_without_a_shell() {
    let argv = [
        "-c".to_owned(),
        "printf out; printf err >&2; exit 3".to_owned(),
    ];
    let reply = SystemSsh.run("sh", &argv).ok();
    assert_eq!(reply, Some(SshReply::new(Some(3), "out", "err")));
}

#[test]
fn the_system_transport_fails_on_a_missing_program() {
    let missing = "cgmr-remote-test-no-such-program";
    assert!(SystemSsh.run(missing, &[]).is_err());
}

fn answer(code: Option<i32>, stdout: &str, stderr: &str) -> Round {
    let mut fake = Fake::answering(SshReply::new(code, stdout, stderr));
    round(&mut fake).unwrap_or_else(|err| unreachable!("{err}"))
}

fn refusal(code: Option<i32>, stdout: &str, stderr: &str) -> RemoteError {
    let mut fake = Fake::answering(SshReply::new(code, stdout, stderr));
    round(&mut fake)
        .err()
        .unwrap_or_else(|| unreachable!("no error"))
}

fn retry_text(round: Round) -> String {
    match round {
        Round::Retry(reason) => reason.as_str().to_owned(),
        Round::Report(_) => unreachable!("not a retry"),
    }
}

#[test]
fn an_unreachable_host_retries_with_ssh_s_last_line() {
    let stderr = "Warning: x\nssh: connect to host box port 22: \
                  Connection refused\n";
    let text = retry_text(answer(Some(255), "", stderr));
    let want = "ssh box: ssh: connect to host box port 22: Connection refused";
    assert_eq!(text, want);
}

#[test]
fn a_silent_ssh_failure_retries_naming_the_exit() {
    assert_eq!(retry_text(answer(Some(255), "", "")), "ssh box: exit 255");
}

#[test]
fn a_killed_ssh_retries() {
    let text = retry_text(answer(None, "", ""));
    assert_eq!(text, "ssh box: killed by a signal");
}

#[test]
fn a_missing_remote_cgmr_is_exit_69() {
    let err = refusal(Some(127), "", "sh: cgmr: command not found\n");
    assert_eq!(err.exit_code(), 69);
    assert_eq!(err.to_string(), "no cgmr on box's PATH");
}

#[test]
fn a_remote_usage_error_is_exit_64_with_its_words() {
    let err = refusal(Some(64), "", "cgmr: --load: not a number\n");
    assert_eq!(err.exit_code(), 64);
    assert_eq!(err.to_string(), "remote cgmr: cgmr: --load: not a number");
}

#[test]
fn a_reply_without_json_says_what_stderr_said() {
    let err = refusal(Some(69), "", "cgmr: --gpu: no source on this OS\n");
    assert_eq!(err.exit_code(), 69);
    let text = err.to_string();
    assert!(
        text.ends_with("(cgmr: --gpu: no source on this OS)"),
        "{text}"
    );
}

#[test]
fn a_silent_remote_usage_error_still_says_so() {
    let err = refusal(Some(64), "", "");
    assert_eq!(err.to_string(), "remote cgmr: usage error");
}

#[test]
fn a_garbled_reply_is_exit_69() {
    assert_eq!(refusal(Some(1), "{oops", "").exit_code(), 69);
}

#[test]
fn ssh_that_cannot_start_is_exit_69() {
    let mut fake = Fake::answering(SshReply::default());
    fake.reply = Err(io::Error::new(io::ErrorKind::NotFound, "no ssh"));
    let err = round(&mut fake).err();
    assert_eq!(
        err.map(|e| e.to_string()),
        Some("cannot start ssh: no ssh".into())
    );
}
