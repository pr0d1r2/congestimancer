use std::path::Path;

use super::{MemSignal, Os};
use crate::signal::{Measure, Root, Sample, Signal, SignalId};

/// The fixture tree `tests/fixtures/mem/<case>`, laid out like `/`.
fn root(case: &str) -> Root {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Root::new(dir.join("tests/fixtures/mem").join(case))
}

/// A mem signal over fixture `case` reading `os`'s sources.
fn on(case: &str, os: Os) -> MemSignal {
    MemSignal::on(root(case), os)
}

/// The reason of an unsupported sample, if it is one.
fn reason(sample: Sample) -> Option<String> {
    match sample {
        Sample::Unsupported(reason) => Some(reason.as_str().to_owned()),
        Sample::Gauge(_) | Sample::Counter(_) => None,
    }
}

#[test]
fn the_host_os_is_picked_by_new() {
    let want = if cfg!(target_os = "macos") {
        Os::Macos
    } else {
        Os::Linux
    };
    assert_eq!(MemSignal::new(root("linux")).os, want);
}

#[test]
fn a_signal_of_another_domain_is_unsupported() {
    assert_eq!(
        reason(on("linux", Os::Linux).sample(SignalId::Load)),
        Some("load is not a mem signal".to_owned())
    );
}

/// Whether `sample` is a bytes gauge: live memory belongs to the
/// machine, so its value is never asserted.
fn is_bytes(sample: &Sample) -> bool {
    matches!(sample, Sample::Gauge(Measure::Bytes(_)))
}

#[test]
fn mem_is_available_bytes_on_linux_sources() {
    let sample = on("linux", Os::Linux).sample(SignalId::Mem);
    assert!(is_bytes(&sample), "{sample:?}");
}

#[test]
fn mem_is_available_bytes_on_macos_sources() {
    let sample = on("linux", Os::Macos).sample(SignalId::Mem);
    assert!(is_bytes(&sample), "{sample:?}");
}
