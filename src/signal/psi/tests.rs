use std::path::Path;
use std::time::Duration;

use super::{PsiError, PsiLine, parse_psi};
use crate::signal::Root;
use crate::units::Percent;

/// The text of a fixture file under `tests/fixtures/signal/`.
fn fixture(path: &str) -> String {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = Root::new(dir.join("tests/fixtures/signal"));
    std::fs::read_to_string(root.join(path)).unwrap_or_default()
}

/// A percent from hundredths.
fn pct(hundredths: u16) -> Percent {
    Percent::from_hundredths(hundredths).unwrap_or_default()
}

/// A PSI line from hundredths and microseconds.
fn line(avg: [u16; 3], micros: u64) -> PsiLine {
    let [avg10, avg60, avg300] = avg.map(pct);
    let total = Duration::from_micros(micros);
    PsiLine {
        avg10,
        avg60,
        avg300,
        total,
    }
}

#[test]
fn proc_cpu_pressure_has_some_and_full() {
    let psi = parse_psi(&fixture("root/proc/pressure/cpu"));
    let psi = psi.unwrap_or_else(|e| unreachable!("{e}"));
    assert_eq!(psi.some, line([153, 87, 40], 12_345_678));
    assert_eq!(psi.full, Some(line([0, 0, 0], 0)));
}

#[test]
fn memory_and_io_pressure_parse_alike() {
    let mem = parse_psi(&fixture("root/proc/pressure/memory"));
    let full = mem.map(|p| p.full);
    assert_eq!(full, Ok(Some(line([475, 200, 91], 123_456_789))));
    let io = parse_psi(&fixture("root/proc/pressure/io"));
    assert_eq!(io.map(|p| p.some.avg10), Ok(Percent::FULL));
}

#[test]
fn an_old_kernel_cpu_file_has_no_full_line() {
    let psi = parse_psi(&fixture("psi/cpu-some-only"));
    assert_eq!(
        psi.map(|p| (p.some.total, p.full)),
        Ok((Duration::from_micros(4242), None))
    );
}

/// Lines that are not `some|full avg10=P avg60=P avg300=P total=N`.
const MALFORMED: [&str; 9] = [
    "half avg10=0.00 avg60=0.00 avg300=0.00 total=0",
    "some avg10=0.00 avg60=0.00 avg300=0.00",
    "some avg10=0.00 avg60=0.00 avg300=0.00 total=0 extra=1",
    "some avg60=0.00 avg10=0.00 avg300=0.00 total=0",
    "some avg10=100.01 avg60=0.00 avg300=0.00 total=0",
    "some avg10=-1.00 avg60=0.00 avg300=0.00 total=0",
    "some avg10=0.00 avg60=0.00 avg300=0.00 total=x",
    "some avg10=0.00 avg60=0.00 avg300=0.00 total=-1",
    "some",
];

#[test]
fn malformed_lines_are_named() {
    for text in MALFORMED {
        let want = PsiError::Malformed {
            line: text.to_owned(),
        };
        assert_eq!(parse_psi(text), Err(want), "{text}");
    }
}

#[test]
fn some_is_required_once_and_full_at_most_once() {
    let some = "some avg10=0.00 avg60=0.00 avg300=0.00 total=0\n";
    let full = "full avg10=0.00 avg60=0.00 avg300=0.00 total=0\n";
    assert_eq!(parse_psi(""), Err(PsiError::MissingSome));
    assert_eq!(parse_psi(full), Err(PsiError::MissingSome));
    let twice = parse_psi(&format!("{some}{some}"));
    assert_eq!(twice, Err(PsiError::Duplicate { kind: "some" }));
    let fulls = parse_psi(&format!("{some}{full}{full}"));
    assert_eq!(fulls, Err(PsiError::Duplicate { kind: "full" }));
    assert!(parse_psi(&format!("\n{full}\n{some}\n")).is_ok());
}

#[test]
fn errors_read_as_sentences() {
    let bad = PsiError::Malformed { line: "x".into() };
    assert_eq!(bad.to_string(), "malformed PSI line: x");
    assert_eq!(
        PsiError::MissingSome.to_string(),
        "PSI text has no `some` line"
    );
    let twice = PsiError::Duplicate { kind: "full" };
    assert_eq!(twice.to_string(), "PSI text has two `full` lines");
}
