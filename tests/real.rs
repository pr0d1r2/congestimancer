//! Real-machine suite (tests:V4): `cgmr status --json` on a test host must
//! give plausible readings: in range for their unit, never exact values.
//! Every test is `#[ignore]`d; test hosts run them with
//! `cargo test -- --ignored` before a release (nix:V20).

use std::error::Error;

type Outcome = Result<(), Box<dyn Error>>;

/// One signal row of `status --json`, as text.
#[derive(Debug)]
struct Row {
    id: String,
    value: String,
    unit: String,
}

/// The text of `"key":` up to the next `,"`, quotes trimmed.
fn field(segment: &str, key: &str) -> String {
    let after = segment.split_once(&format!("\"{key}\":")).map(|p| p.1);
    let text = after.unwrap_or_default().split(",\"").next();
    text.unwrap_or_default().trim_matches('"').to_owned()
}

/// The signal rows of the live machine's `status --json`.
fn status() -> Result<Vec<Row>, Box<dyn Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("cgmr")?;
    cmd.env_remove("CGMR_FIXTURE_ROOT");
    let out = cmd
        .args(["status", "--json", "--interval", "500ms"])
        .output()?;
    assert!(out.status.success());
    let json = String::from_utf8(out.stdout)?;
    let rows = json.split("{\"id\":\"").skip(1).map(|s| Row {
        id: s.split('"').next().unwrap_or_default().to_owned(),
        value: field(s, "value"),
        unit: field(s, "unit"),
    });
    Ok(rows.collect())
}

/// Whether `text` is a number in `[low, high]`.
fn within(text: &str, low: f64, high: f64) -> bool {
    text.parse::<f64>()
        .is_ok_and(|v| v.is_finite() && (low..=high).contains(&v))
}

/// Whether a supported reading is plausible for its unit.
fn plausible(row: &Row) -> bool {
    let v = row.value.as_str();
    match row.unit.as_str() {
        "percent" => within(v, 0.0, 100.0),
        "number" | "bytes" | "bytes/s" | "count" | "seconds" => {
            within(v, 0.0, f64::MAX)
        }
        "celsius" => within(v, -40.0, 150.0),
        "flag" => matches!(v, "true" | "false"),
        "level" => matches!(v, "normal" | "warning" | "critical"),
        "clock" => v.len() == 5 && v.as_bytes().get(2) == Some(&b':'),
        _ => false,
    }
}

#[test]
#[ignore = "reads the live machine; run on test hosts (nix:V20)"]
fn every_supported_reading_is_plausible() -> Outcome {
    let rows = status()?;
    let read: Vec<&Row> = rows.iter().filter(|r| r.value != "null").collect();
    assert!(!read.is_empty());
    let odd: Vec<String> = read
        .iter()
        .filter(|r| !plausible(r))
        .map(|r| format!("{r:?}"))
        .collect();
    assert!(odd.is_empty(), "{}", odd.join("\n"));
    Ok(())
}

#[test]
#[ignore = "reads the live machine; run on test hosts (nix:V20)"]
fn the_core_signals_are_read_on_every_host() -> Outcome {
    let rows = status()?;
    for id in [
        "load",
        "run-queue",
        "cpu-idle",
        "mem",
        "disk",
        "fds",
        "pids",
    ] {
        let row = rows.iter().find(|r| r.id == id);
        assert!(row.is_some_and(|r| r.value != "null"), "{id}: {row:?}");
    }
    Ok(())
}

#[test]
#[ignore = "reads the live machine; run on test hosts (nix:V20)"]
fn memory_and_disk_are_machine_sized() -> Outcome {
    let rows = status()?;
    let value =
        |id: &str| rows.iter().find(|r| r.id == id).map(|r| r.value.clone());
    let mib = 1024.0 * 1024.0;
    assert!(within(&value("mem").unwrap_or_default(), 16.0 * mib, 1e16));
    assert!(within(&value("disk").unwrap_or_default(), 1.0, 1e18));
    Ok(())
}
