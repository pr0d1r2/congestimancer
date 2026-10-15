use super::{CpuTimes, Loads, ProcStat, parse_loadavg, parse_stat};
use crate::cpu::CpuError;
use crate::units::Decimal;

/// A decimal of `thousandths` thousandths.
const fn dec(thousandths: u64) -> Decimal {
    Decimal::from_thousandths(thousandths)
}

#[test]
fn loadavg_gives_the_three_averages() {
    assert_eq!(
        parse_loadavg("0.52 1.58 12.05 3/1234 5678\n"),
        Ok(Loads {
            one: dec(520),
            five: dec(1_580),
            fifteen: dec(12_050),
        })
    );
}

#[test]
fn loadavg_with_two_averages_is_malformed() {
    assert_eq!(
        parse_loadavg("0.52 1.58\n"),
        Err(CpuError::Malformed {
            file: "/proc/loadavg",
            line: "0.52 1.58".to_owned()
        })
    );
}

#[test]
fn loadavg_with_a_garbled_average_is_malformed() {
    assert_eq!(
        parse_loadavg("0.52 lots 1.0 1/2 3\n"),
        Err(CpuError::Malformed {
            file: "/proc/loadavg",
            line: "0.52 lots 1.0 1/2 3".to_owned()
        })
    );
}

/// A `/proc/stat` like a 2-CPU Linux host's.
const STAT: &str = "\
cpu  100 2 30 400 5 6 7 8 9 10
cpu0 50 1 15 200 2 3 3 4 4 5
cpu1 50 1 15 200 3 3 4 4 5 5
intr 12345 0 0
ctxt 999
procs_running 3
procs_blocked 1
";

/// The times of [`STAT`]'s `cpu` line.
const TIMES: CpuTimes = CpuTimes {
    user: 100,
    nice: 2,
    system: 30,
    idle: 400,
    iowait: 5,
    irq: 6,
    softirq: 7,
    steal: 8,
};

#[test]
fn stat_gives_cpu_times_and_procs_running() {
    assert_eq!(
        parse_stat(STAT),
        Ok(ProcStat {
            times: TIMES,
            procs_running: 3
        })
    );
}

#[test]
fn stat_from_an_old_kernel_has_zero_for_missing_columns() {
    let stat = parse_stat("cpu 1 2 3 4\nprocs_running 1\n");
    let times = CpuTimes {
        user: 1,
        nice: 2,
        system: 3,
        idle: 4,
        ..CpuTimes::default()
    };
    assert_eq!(stat.map(|s| s.times), Ok(times));
}

#[test]
fn stat_without_procs_running_is_missing_it() {
    assert_eq!(
        parse_stat("cpu 1 2 3 4\n"),
        Err(CpuError::Missing {
            file: "/proc/stat",
            key: "procs_running"
        })
    );
}

#[test]
fn stat_without_a_cpu_line_is_missing_it() {
    assert_eq!(
        parse_stat("cpu0 1 2 3 4\nprocs_running 1\n"),
        Err(CpuError::Missing {
            file: "/proc/stat",
            key: "cpu"
        })
    );
}

#[test]
fn stat_with_a_short_cpu_line_is_malformed() {
    assert_eq!(
        parse_stat("cpu 1 2 3\nprocs_running 1\n"),
        Err(CpuError::Malformed {
            file: "/proc/stat",
            line: "cpu 1 2 3".to_owned()
        })
    );
}

#[test]
fn stat_with_a_garbled_procs_running_is_malformed() {
    assert_eq!(
        parse_stat("cpu 1 2 3 4\nprocs_running many\n"),
        Err(CpuError::Malformed {
            file: "/proc/stat",
            line: "procs_running many".to_owned()
        })
    );
}

#[test]
fn cpu_total_sums_the_eight_states() {
    assert_eq!(TIMES.total(), Some(558));
}

#[test]
fn cpu_idle_total_counts_iowait_as_idle() {
    assert_eq!(TIMES.idle_total(), Some(405));
}

#[test]
fn cpu_totals_past_64_bits_are_none() {
    let times = CpuTimes {
        idle: u64::MAX,
        iowait: 1,
        ..CpuTimes::default()
    };
    assert_eq!((times.total(), times.idle_total()), (None, None));
}
