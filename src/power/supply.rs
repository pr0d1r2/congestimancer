//! Power supplies: Linux `/sys/class/power_supply/*/uevent` text and the
//! macOS IOKit [`PowerState`], as one [`Supply`] list (src/power:T5, V3,
//! V4).

use super::PowerError;
use crate::sys::PowerState;
use crate::units::Percent;

/// What a power supply is, from `POWER_SUPPLY_TYPE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SupplyKind {
    /// Wall power: an AC adapter.
    Mains,
    /// A battery: the machine's own, or a peripheral's.
    Battery,
    /// A USB or USB-C power port.
    Usb,
    /// An uninterruptible power supply.
    Ups,
    /// Anything else the kernel names.
    Other,
}

/// One power supply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Supply {
    /// What it is.
    pub kind: SupplyKind,
    /// Whether it delivers power now (`POWER_SUPPLY_ONLINE`), if known.
    pub online: Option<bool>,
    /// Whether it is there at all: an empty battery bay is not.
    pub present: bool,
    /// Charge, from `CAPACITY`, else `ENERGY_NOW` of `ENERGY_FULL`, else
    /// `CHARGE_NOW` of `CHARGE_FULL`.
    pub capacity: Option<Percent>,
    /// A peripheral's supply (`POWER_SUPPLY_SCOPE=Device`): a mouse
    /// battery, never the machine's.
    pub device: bool,
}

impl Supply {
    /// Whether this is a battery that powers the machine.
    #[must_use]
    pub const fn is_system_battery(&self) -> bool {
        let _ = self;
        false
    }
}

/// A `power_supply` `uevent` file: `KEY=value` lines.
///
/// # Errors
///
/// [`PowerError::Malformed`] when there is no `POWER_SUPPLY_TYPE`.
pub fn parse_supply_uevent(text: &str) -> Result<Supply, PowerError> {
    Err(PowerError::malformed("uevent", text))
}

/// The supplies IOKit reports on macOS: wall power, and the internal
/// battery when there is one.
#[must_use]
pub fn supplies_of(state: &PowerState) -> Vec<Supply> {
    let _ = state;
    Vec::new()
}

/// Whether the machine runs on mains power: a mains or USB supply is
/// online, or the machine has no battery at all, as a desktop
/// (src/power:V3).
#[must_use]
pub fn on_mains(supplies: &[Supply]) -> bool {
    let _ = supplies;
    false
}

/// The mean charge of the machine's batteries that report one; `None`
/// when none does (src/power:V4).
#[must_use]
pub fn battery_charge(supplies: &[Supply]) -> Option<Percent> {
    let _ = supplies;
    None
}

#[cfg(test)]
mod tests;
