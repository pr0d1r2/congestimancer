use super::{
    DirtyPages, SwapPages, parse_meminfo, parse_page_size, parse_vmstat,
};
use crate::mem::MemError;
use crate::units::Bytes;

#[test]
fn vmstat_gives_pages_swapped_in_and_out() {
    let text = "nr_free_pages 1000\npswpin 12\npgfault 9\npswpout 34\n";
    assert_eq!(
        parse_vmstat(text),
        Ok(SwapPages {
            pages_in: 12,
            pages_out: 34
        })
    );
}

#[test]
fn vmstat_without_pswpout_is_missing_it() {
    assert_eq!(
        parse_vmstat("pswpin 12\n"),
        Err(MemError::Missing {
            file: "/proc/vmstat",
            key: "pswpout"
        })
    );
}

#[test]
fn vmstat_with_a_garbled_line_is_malformed() {
    assert_eq!(
        parse_vmstat("pswpin lots\npswpout 1\n"),
        Err(MemError::Malformed {
            file: "/proc/vmstat",
            line: "pswpin lots".to_owned()
        })
    );
}

#[test]
fn vmstat_skips_blank_lines() {
    let pages = parse_vmstat("\npswpin 1\n\npswpout 2\n");
    assert_eq!(pages.map(|p| p.pages_out), Ok(2));
}

#[test]
fn swap_bytes_are_pages_times_page_size() {
    let pages = SwapPages {
        pages_in: 12,
        pages_out: 34,
    };
    assert_eq!(pages.bytes(4096), Some(46 * 4096));
}

#[test]
fn swap_bytes_past_64_bits_are_none() {
    let pages = SwapPages {
        pages_in: u64::MAX,
        pages_out: 1,
    };
    assert_eq!(pages.bytes(1), None);
    let pages = SwapPages {
        pages_in: u64::MAX / 2,
        pages_out: 0,
    };
    assert_eq!(pages.bytes(4096), None);
}

#[test]
fn page_size_is_the_smallest_kernel_page_size() {
    let text =
        "KernelPageSize:     2048 kB\nRss: 8 kB\nKernelPageSize:  4 kB\n";
    assert_eq!(parse_page_size(text), Ok(4096));
}

#[test]
fn page_size_without_kernel_page_size_is_missing() {
    assert_eq!(
        parse_page_size("Rss: 8 kB\n"),
        Err(MemError::Missing {
            file: "/proc/self/smaps",
            key: "KernelPageSize"
        })
    );
}

#[test]
fn page_size_in_another_unit_is_malformed() {
    assert_eq!(
        parse_page_size("KernelPageSize: 4 MB\n"),
        Err(MemError::Malformed {
            file: "/proc/self/smaps",
            line: "KernelPageSize: 4 MB".to_owned()
        })
    );
}

#[test]
fn a_page_size_past_64_bits_is_malformed() {
    let line = format!("KernelPageSize: {} kB", u64::MAX);
    assert!(matches!(
        parse_page_size(&line),
        Err(MemError::Malformed { .. })
    ));
}

#[test]
fn meminfo_gives_dirty_and_writeback_bytes() {
    let text = "MemFree: 9 kB\nDirty:  12 kB\nWriteback:   3 kB\n";
    assert_eq!(
        parse_meminfo(text),
        Ok(DirtyPages {
            dirty: Bytes(12 * 1024),
            writeback: Bytes(3 * 1024)
        })
    );
}

#[test]
fn meminfo_without_writeback_is_missing_it() {
    assert_eq!(
        parse_meminfo("Dirty: 1 kB\n"),
        Err(MemError::Missing {
            file: "/proc/meminfo",
            key: "Writeback"
        })
    );
}

#[test]
fn meminfo_with_a_bare_number_is_malformed() {
    assert_eq!(
        parse_meminfo("Dirty: 1\nWriteback: 0 kB\n"),
        Err(MemError::Malformed {
            file: "/proc/meminfo",
            line: "Dirty: 1".to_owned()
        })
    );
}

#[test]
fn writeback_does_not_match_writeback_tmp() {
    let text = "WritebackTmp: 7 kB\nDirty: 0 kB\nWriteback: 1 kB\n";
    let pages = parse_meminfo(text).map(|p| p.writeback);
    assert_eq!(pages, Ok(Bytes(1024)));
}

#[test]
fn the_dirty_backlog_is_dirty_plus_writeback() {
    let pages = DirtyPages {
        dirty: Bytes(5),
        writeback: Bytes(7),
    };
    assert_eq!(pages.backlog(), Some(Bytes(12)));
}

#[test]
fn a_dirty_backlog_past_64_bits_is_none() {
    let pages = DirtyPages {
        dirty: Bytes(u64::MAX),
        writeback: Bytes(1),
    };
    assert_eq!(pages.backlog(), None);
}
