use std::path::Path;

use super::{amd_gpus, parse_busy_percent, parse_vram};
use crate::gpu::{Gpu, GpuError};
use crate::signal::Root;
use crate::units::{Bytes, Percent};

/// The fixture tree `tests/fixtures/gpu/<case>`, laid out like `/`.
fn root(case: &str) -> Root {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Root::new(dir.join("tests/fixtures/gpu").join(case))
}

/// `n` whole percent.
fn pct(n: u16) -> Option<Percent> {
    n.checked_mul(100).and_then(Percent::from_hundredths)
}

#[test]
fn busy_percent_is_a_whole_percent() {
    assert_eq!(parse_busy_percent("42\n").ok(), pct(42));
    assert_eq!(parse_busy_percent("0").ok(), pct(0));
    assert_eq!(parse_busy_percent("100\n").ok(), pct(100));
}

#[test]
fn busy_percent_refuses_past_100_and_words() {
    for text in ["101\n", "busy\n", "-1", "", "4.5"] {
        assert_eq!(
            parse_busy_percent(text),
            Err(GpuError::malformed("gpu_busy_percent", text)),
            "{text:?}"
        );
    }
}

#[test]
fn vram_is_bytes() {
    let file = "mem_info_vram_total";
    assert_eq!(parse_vram(file, "8589934592\n"), Ok(Bytes(8_589_934_592)));
    assert_eq!(
        parse_vram(file, "lots"),
        Err(GpuError::malformed(file, "lots"))
    );
}

#[test]
fn cards_are_read_by_number_skipping_connectors_and_others() {
    let gib = 1 << 30;
    assert_eq!(
        amd_gpus(&root("amd")),
        [
            Gpu::new(pct(42), Some(Bytes(6 * gib))),
            Gpu::new(pct(90), None),
            Gpu::new(pct(7), Some(Bytes(gib / 2))),
        ]
    );
}

#[test]
fn unparsed_files_and_used_past_total_leave_fields_empty() {
    assert_eq!(
        amd_gpus(&root("amd-odd")),
        [Gpu::new(None, None), Gpu::new(None, None)]
    );
}

#[test]
fn no_amd_card_or_no_drm_class_is_no_gpu() {
    assert_eq!(amd_gpus(&root("none")), []);
    assert_eq!(amd_gpus(&root("missing")), []);
}
