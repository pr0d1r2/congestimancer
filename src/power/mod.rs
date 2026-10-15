//! `src/power`: thermal throttling, temperature, AC, battery, low power.
//!
//! Composes only (src:C): each part lives in its own submodule, and the
//! public surface is re-exported here and again from the crate root
//! (src:V3). Linux files are read through `crate::sys` under a
//! [`Root`](crate::signal::Root), so tests read `tests/fixtures/power/`;
//! macOS goes through `crate::sys` notify and IOKit calls.

mod error;
mod ppd;
mod signal;
mod supply;
mod thermal;

pub use error::PowerError;
pub use ppd::{PPD_STATE, parse_ppd_profile};
pub use signal::PowerSignal;
pub use supply::{
    Supply, SupplyKind, battery_charge, on_mains, parse_supply_uevent,
    supplies_of,
};
pub use thermal::{
    is_cpu_cooling, parse_cpu_list, parse_millicelsius, parse_sysfs_u64,
};
