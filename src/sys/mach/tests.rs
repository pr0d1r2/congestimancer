use std::path::Path;

use super::{
    CpuTicks, PsetLoad, cpu_ticks, decode_cpu_ticks, decode_pset_load,
    decode_vm_stats, pset_load, vm_stats,
};
use crate::sys::raw::decode_hex;

/// Raw words of a mach call captured on an M4 (10 CPU) macOS 26 host.
fn captured(name: &str) -> Vec<u8> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = dir.join("tests/fixtures/sys/macos/mach").join(name);
    let text = std::fs::read_to_string(path).unwrap_or_default();
    decode_hex(&text).unwrap_or_default()
}

const fn ticks(user: u64, system: u64, idle: u64) -> CpuTicks {
    CpuTicks {
        user,
        system,
        idle,
        nice: 0,
    }
}

#[test]
fn cpu_ticks_decode_four_states_a_cpu() {
    let all = decode_cpu_ticks(&captured("host_processor_info.cpu_load"));
    assert_eq!(all.len(), 10);
    let first = ticks(24_867_125, 12_021_617, 88_032_481);
    let last = ticks(14_435_392, 6_889_360, 104_322_854);
    assert_eq!(all.first(), Some(&first));
    assert_eq!(all.last(), Some(&last));
}

#[test]
fn a_partial_cpu_is_dropped() {
    let mut raw = captured("host_processor_info.cpu_load");
    raw.truncate(16 + 12);
    assert_eq!(decode_cpu_ticks(&raw).len(), 1);
}

#[test]
fn vm_stats_decode_page_gauges_by_field_offset() {
    let v = decode_vm_stats(&captured("host_statistics64.vm_info64"));
    let v = v.unwrap_or_default();
    let gauges = [v.free, v.active, v.inactive, v.wired, v.purgeable];
    assert_eq!(gauges, [4238, 165_436, 163_181, 476_275, 1]);
    let more = [v.speculative, v.compressor, v.external, v.internal];
    assert_eq!(more, [754, 199_317, 93_200, 236_171]);
}

#[test]
fn vm_stats_decode_page_counters_by_field_offset() {
    let v = decode_vm_stats(&captured("host_statistics64.vm_info64"));
    let v = v.unwrap_or_default();
    let counters = [v.pageins, v.pageouts, v.swapins, v.swapouts];
    assert_eq!(counters, [325_761_608, 2_492_380, 36_731_247, 39_236_757]);
}

#[test]
fn a_short_vm_reply_is_none() {
    let mut raw = captured("host_statistics64.vm_info64");
    raw.truncate(143);
    assert_eq!(decode_vm_stats(&raw), None);
}

#[test]
fn pset_load_decodes_four_integers() {
    let load =
        decode_pset_load(&captured("processor_set_statistics.load_info"));
    assert_eq!(
        load,
        Some(PsetLoad {
            task_count: 491,
            thread_count: 2575,
            load_average: 9374,
            mach_factor: 3352,
        })
    );
    assert_eq!(decode_pset_load(&[0; 15]), None);
}

#[cfg(not(target_os = "macos"))]
#[test]
fn other_oses_are_unsupported() {
    use std::io::ErrorKind::Unsupported;

    assert_eq!(cpu_ticks().err().map(|e| e.kind()), Some(Unsupported));
    assert_eq!(vm_stats().err().map(|e| e.kind()), Some(Unsupported));
    assert_eq!(pset_load().err().map(|e| e.kind()), Some(Unsupported));
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "live machine (src:V4): run with --ignored"]
fn live_mach_calls_read() {
    let cpus = std::thread::available_parallelism().map_or(0, usize::from);
    assert_eq!(cpu_ticks().map(|t| t.len()).ok(), Some(cpus));
    assert!(vm_stats().is_ok_and(|v| v.wired > 0));
    assert!(pset_load().is_ok_and(|p| p.thread_count > 0));
    assert!(pset_load().is_ok(), "the cached pset port is reused");
}
