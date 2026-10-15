//! `src/limits`: open files & processes vs system maximum.
//!
//! Composes only (src:C): parsers in `parse`, the gates in `signal`.
//! Linux files are read through `crate::sys` under a
//! [`Root`](crate::signal::Root), so tests read `tests/fixtures/limits/`;
//! macOS reads sysctls through `crate::sys`.

mod error;
mod parse;
mod signal;

pub use error::LimitsError;
pub use parse::{
    FileNr, limit_share, parse_file_nr, parse_loadavg_tasks, parse_pid_max,
};
pub use signal::LimitsSignal;
