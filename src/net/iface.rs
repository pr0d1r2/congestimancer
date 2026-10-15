//! [`parse_net_iface`]: the `--net-iface I` value (src/net:T6, V6).

use super::NetError;
use crate::units::{Name, parse_name};

/// Parses `--net-iface I`: `I` is a [`Name`] (src/units:V13), so it is
/// safe as the path segment `/sys/class/net/I` (src/net:V6).
///
/// # Errors
///
/// [`NetError::BadIface`] carrying the [`UnitError`] that names the
/// input and the form expected: a usage error, exit 64.
///
/// [`UnitError`]: crate::units::UnitError
pub fn parse_net_iface(s: &str) -> Result<Name, NetError> {
    parse_name(s).map_err(|error| NetError::BadIface { error })
}

#[cfg(test)]
mod tests;
