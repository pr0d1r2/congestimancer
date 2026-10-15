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
        let battery = matches!(self.kind, SupplyKind::Battery);
        battery && self.present && !self.device
    }
}

/// A `power_supply` `uevent` file: `KEY=value` lines.
///
/// # Errors
///
/// [`PowerError::Malformed`] when there is no `POWER_SUPPLY_TYPE`.
pub fn parse_supply_uevent(text: &str) -> Result<Supply, PowerError> {
    let kind = field(text, "TYPE")
        .map(kind_of)
        .ok_or_else(|| PowerError::malformed("uevent", text))?;
    let flag = |key| field(text, key).map(|value| value.trim() == "1");
    Ok(Supply {
        kind,
        online: flag("ONLINE"),
        present: flag("PRESENT").unwrap_or(true),
        capacity: capacity(text),
        device: field(text, "SCOPE").is_some_and(|s| s.trim() == "Device"),
    })
}

/// The value of `POWER_SUPPLY_<key>` in a `uevent` text.
fn field<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines().find_map(|line| {
        line.strip_prefix("POWER_SUPPLY_")?
            .strip_prefix(key)?
            .strip_prefix('=')
    })
}

/// The kind a `POWER_SUPPLY_TYPE` value names.
fn kind_of(value: &str) -> SupplyKind {
    match value.trim() {
        "Mains" => SupplyKind::Mains,
        "Battery" => SupplyKind::Battery,
        "USB" => SupplyKind::Usb,
        "UPS" => SupplyKind::Ups,
        _ => SupplyKind::Other,
    }
}

/// Charge from `CAPACITY`, else energy or charge now of full.
fn capacity(text: &str) -> Option<Percent> {
    let number = |key| field(text, key)?.trim().parse::<u64>().ok();
    let share = |now, full| {
        let now: u64 = number(now)?;
        now.checked_mul(100)?.checked_div(number(full)?)
    };
    number("CAPACITY")
        .or_else(|| share("ENERGY_NOW", "ENERGY_FULL"))
        .or_else(|| share("CHARGE_NOW", "CHARGE_FULL"))
        .and_then(whole_percent)
}

/// `whole` percent, capped at 100.
fn whole_percent(whole: u64) -> Option<Percent> {
    let whole = u16::try_from(whole.min(100)).ok()?;
    Percent::from_hundredths(whole.checked_mul(100)?)
}

/// The supplies IOKit reports on macOS: wall power, and the internal
/// battery when there is one.
#[must_use]
pub fn supplies_of(state: &PowerState) -> Vec<Supply> {
    let mains = Supply {
        kind: SupplyKind::Mains,
        online: Some(state.external),
        present: true,
        capacity: None,
        device: false,
    };
    let battery = state.battery_percent.map(|percent| Supply {
        kind: SupplyKind::Battery,
        online: None,
        present: true,
        capacity: whole_percent(u64::from(percent)),
        device: false,
    });
    std::iter::once(mains).chain(battery).collect()
}

/// Whether the machine runs on mains power: a mains or USB supply is
/// online, or the machine has no battery at all, as a desktop
/// (src/power:V3).
#[must_use]
pub fn on_mains(supplies: &[Supply]) -> bool {
    let online = supplies.iter().any(|supply| {
        matches!(supply.kind, SupplyKind::Mains | SupplyKind::Usb)
            && supply.online == Some(true)
    });
    online || !supplies.iter().any(Supply::is_system_battery)
}

/// The mean charge of the machine's batteries that report one; `None`
/// when none does (src/power:V4).
#[must_use]
pub fn battery_charge(supplies: &[Supply]) -> Option<Percent> {
    let charges: Vec<u32> = supplies
        .iter()
        .filter(|supply| supply.is_system_battery())
        .filter_map(|supply| supply.capacity)
        .map(|charge| u32::from(charge.hundredths()))
        .collect();
    let count = u32::try_from(charges.len()).ok()?;
    let sum = charges
        .iter()
        .try_fold(0_u32, |sum, &c| sum.checked_add(c))?;
    let mean = u16::try_from(sum.checked_div(count)?).ok()?;
    Percent::from_hundredths(mean)
}

#[cfg(test)]
mod tests;
