use super::write_status;
use crate::report::{LearnedPeak, StatusReport, StatusRow, Style};
use crate::signal::{Measure, Reading, SignalId, Trend};
use crate::units::{Bytes, Decimal, Percent};

fn load(thousandths: u64) -> Reading {
    Reading::Value(Measure::Number(Decimal::from_thousandths(thousandths)))
}

fn table(rows: Vec<StatusRow>, style: Style) -> String {
    let mut report = StatusReport::new("box");
    report.signals = rows;
    let mut out = Vec::new();
    assert!(write_status(&mut out, &report, style).is_ok());
    String::from_utf8(out).unwrap_or_default()
}

#[test]
fn an_empty_report_names_only_the_host() {
    assert_eq!(table(Vec::new(), Style::Plain), "host box\n");
}

#[test]
fn rows_line_up_under_a_header() {
    let rows = vec![
        StatusRow::new(SignalId::Load, load(1_500)),
        StatusRow::new(SignalId::Mem, Reading::Value(Measure::Bytes(Bytes(1))))
            .with_source("cgroup"),
    ];
    let want = "host box\n\
                SIGNAL  SOURCE  VALUE\n\
                load    host    1.5\n\
                mem     cgroup  1B\n";
    assert_eq!(table(rows, Style::Plain), want);
}

#[test]
fn a_trend_gets_its_own_column() {
    let rows = vec![
        StatusRow::new(SignalId::Load, load(12_250))
            .with_trend(Some(Trend::Rising)),
        StatusRow::new(SignalId::RunQueue, load(2_000)),
    ];
    let want = "host box\n\
                SIGNAL     SOURCE  VALUE  TREND\n\
                load       host    12.25  rising\n\
                run-queue  host    2\n";
    assert_eq!(table(rows, Style::Plain), want);
}

#[test]
fn unsupported_signals_are_listed_with_their_reason() {
    let rows = vec![
        StatusRow::new(SignalId::CpuPressure, Reading::unsupported("no PSI")),
        StatusRow::new(SignalId::Load, load(1_000))
            .with_trend(Some(Trend::Flat)),
    ];
    let want = "host box\n\
                SIGNAL        SOURCE  VALUE  TREND\n\
                cpu-pressure  host    unsupported: no PSI\n\
                load          host    1      flat\n";
    assert_eq!(table(rows, Style::Plain), want);
}

#[test]
fn warming_signals_say_so() {
    let rows = vec![StatusRow::new(SignalId::IoBusy, Reading::Warming)];
    assert!(
        table(rows, Style::Plain).ends_with("io-busy  host    warming up\n")
    );
}

#[test]
fn percents_render_with_a_dot() {
    let half = Percent::from_hundredths(5_050).map(Measure::Percent);
    let rows = half
        .map(|m| vec![StatusRow::new(SignalId::CpuIdle, Reading::Value(m))])
        .unwrap_or_default();
    assert!(table(rows, Style::Plain).ends_with("host    50.5%\n"));
}

#[test]
fn colour_paints_the_header_and_odd_values() {
    let rows = vec![
        StatusRow::new(SignalId::Gpu, Reading::unsupported("no GPU")),
        StatusRow::new(SignalId::IoBusy, Reading::Warming),
    ];
    let shown = table(rows, Style::Ansi);
    assert!(shown.starts_with("\u{1b}[1mhost\u{1b}[0m box\n"));
    assert!(shown.contains("\u{1b}[1mSIGNAL   SOURCE  VALUE\u{1b}[0m\n"));
    assert!(shown.contains("\u{1b}[33munsupported: no GPU\u{1b}[0m\n"));
    assert!(shown.contains("\u{1b}[2mwarming up\u{1b}[0m\n"));
}

#[test]
fn learned_peaks_follow_the_table_in_their_own_section() {
    let mut report = StatusReport::new("box");
    report.signals = vec![StatusRow::new(SignalId::Load, load(1_500))];
    report
        .learned
        .push(LearnedPeak::new("gate", Bytes(3 << 30)));
    report
        .learned
        .push(LearnedPeak::new("lint", Bytes(512 << 20)));
    let mut out = Vec::new();
    assert!(write_status(&mut out, &report, Style::Plain).is_ok());
    let text = String::from_utf8(out).unwrap_or_default();
    let tail = text.split("\n\n").nth(1).unwrap_or_default();
    assert_eq!(tail, "learned memory peaks\ngate  3GiB\nlint  512MiB\n");
}

#[test]
fn no_learned_peaks_adds_nothing() {
    assert_eq!(table(Vec::new(), Style::Plain), "host box\n");
}
