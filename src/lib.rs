//! congestimancer: gate commands on machine congestion.
//!
//! This file only re-exports node modules (src:V1, src:V3); every node is a
//! private module whose public surface is re-exported here as it is built.

mod advise;
mod cgroup;
mod cli;
mod cpu;
mod disk;
mod gate;
mod gpu;
mod io;
mod launch;
mod limits;
mod mem;
mod net;
mod power;
mod presence;
mod procs;
mod profile;
mod remote;
mod report;
mod signal;
mod slot;
mod sys;
mod units;
mod wait;

pub use advise::{
    Advice, AdviceAction, AdviceLimit, AdviseRequest, AdviseUnsupported,
    advise, advise_exit_code, advise_readings, cpu_fit, decide, mem_fit,
    write_advice, write_advice_json,
};

pub use cgroup::{
    CGROUP_SIGNALS, Cgroup, CgroupError, CgroupMode, CgroupSignal, CpuMax,
    CpuStat, Dual, Scope, Scoped, Throttling, load_warning, memory_available,
    parse_cgroup_mode, parse_cpu_max, parse_cpu_stat, parse_memory_current,
    parse_memory_max, parse_proc_cgroup,
};

pub use cpu::{
    CpuError, CpuSignal, CpuTimes, LoadWindow, Loads, ProcStat, cpu_count,
    load_per_cpu, parse_load_window, parse_loadavg, parse_stat,
};

pub use disk::{DiskError, DiskSignal, free_inodes, free_space};

pub use gate::{
    Adjust, Checked, Custom, Evaluation, Gate, Gates, Op, Overall, Subject,
    Threshold, Verdict, evaluate, evaluate_once, sample_once,
};

pub use gpu::{
    Gpu, GpuError, GpuSignal, amd_gpus, parse_busy_percent, parse_vram,
    worst_busy, worst_free,
};

pub use io::{
    Direction, DiskStat, IoError, IoSignal, Mount, linux_device, macos_device,
    mount_of, parse_diskstats, parse_io_device, parse_mountinfo,
};

pub use launch::{
    BACKGROUND_NICE, Priority, Slots, Start, after_wait, background, clean_env,
    exec_command, exec_failure_code, launch,
};

pub use limits::{
    FileNr, LimitsError, LimitsSignal, limit_share, parse_file_nr,
    parse_loadavg_tasks, parse_pid_max,
};

pub use mem::{
    DirtyPages, MemError, MemSignal, SwapPages, available_level, level_within,
    mem_per_cpu, mem_unreserved, parse_meminfo, parse_page_size, parse_vmstat,
    pressure_level,
};

pub use net::{
    NetDevError, NetError, NetSignal, RTT_PORT, RttSignal, linux_interfaces,
    parse_if_flags, parse_net_dev, parse_net_iface, selected,
};

pub use power::{
    PPD_STATE, PowerError, PowerSignal, Supply, SupplyKind, battery_charge,
    is_cpu_cooling, on_mains, parse_cpu_list, parse_millicelsius,
    parse_ppd_profile, parse_supply_uevent, parse_sysfs_u64, supplies_of,
};

pub use presence::{PresenceSignal, SystemWallClock, WallClock};

pub use procs::{
    Glob, LiveTable, Proc, ProcLimit, ProcMatch, ProcessTable, ProcsError,
    ProcsSignal, count_by_name, count_by_user, lineage, parse_proc_limit,
};

pub use profile::{
    Arity, At, Catalog, Origin, PROJECT_FILE, Places, Profile, ProfileError,
    ProfileFile, ReadText, Switch, SwitchFault, SwitchTable, discover,
    parse_profiles, render_profiles,
};

pub use remote::{
    Host, RemoteError, RemoteGate, RemoteReport, RemoteSignal, RemoteVerdict,
    Round, SSH, SshReply, SystemSsh, Transport, check_remote, parse_host,
    parse_reply, shell_quote, ssh_argv,
};

pub use report::{
    Output, Paint, SCHEMA_VERSION, StatusReport, StatusRow, Stream, Streams,
    Style, Warnings, block_reasons, heartbeat_line, json_string, switch_name,
    write_json, write_status, write_verbose,
};

pub use signal::{
    Clock, Counter, Level, Measure, Psi, PsiError, PsiLine, Reading, Reason,
    Root, Sample, Sampled, Sampler, Signal, SignalId, SignalSet, Source,
    SystemClock, Trend, Trends, parse_psi, refresh,
};

pub use slot::{
    DirFacts, HELD_SLOTS_ENV, LockDir, Release, Reservation, SLOT_ORDER,
    SlotError, SlotGuard, SlotPlan, SlotStep, StartLimits, StartRate, Ticket,
    Waiter, admit_start, claim_slot, is_held, live_holders, lock_dir_path,
    mark_held, parse_start_rate, record_start, reserved_total, slot_key,
    unsafe_reason,
};

pub use sys::{
    BlockCounters, BlockDevice, BlockStats, CpuTicks, FsStats, GpuUsage,
    InterfaceCounters, IoPriority, LOW_POWER_MODE, LinkCounters, LoadAverage,
    Memory, MountSource, NvmlGpu, PowerState, ProcessInfo, PropValue, Props,
    PsetLoad, SYSTEM_BUS_SOCKET, Snapshot, SwapUsage, THERMAL_PRESSURE,
    VmStats, block_devices, capture, cpu_ticks, current_uid, device_of,
    gpu_usage, hid_idle_ns, interface_counters, io_priority, list_dir,
    local_time, mount_source, nice, notify_state, nvml_gpus,
    physical_disk_stats, power_state, pset_load, read_text, set_inheritable,
    set_io_priority, set_nice, statvfs, swap_usage, sysctl_int,
    system_bus_connect, tcp_connect_time, try_lock_exclusive, unlock, user_id,
    user_name, vm_stats,
};

pub use units::{
    Bytes, Celsius, ClockTime, ClockWindow, Count, Decimal, Name, Percent,
    Rate, UnitError, parse_bytes, parse_celsius, parse_clock_window,
    parse_count, parse_decimal, parse_duration, parse_name, parse_percent,
    parse_rate, render_duration,
};

pub use wait::{
    Admit, DEFAULT_INTERVAL, MIN_INTERVAL, Open, Outcome, Pacing, Rig,
    jitter_delay, poll,
};
