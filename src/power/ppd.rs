//! power-profiles-daemon's saved state, `state.ini`: the active profile
//! (src/power:T7, V5).

use super::PowerError;

/// Where power-profiles-daemon keeps its state.
pub const PPD_STATE: &str = "/var/lib/power-profiles-daemon/state.ini";

/// The `Profile` of the `[State]` section of `state.ini`, such as
/// `balanced` or `power-saver`.
///
/// # Errors
///
/// [`PowerError::Malformed`] when `[State]` has no non-empty `Profile`.
pub fn parse_ppd_profile(text: &str) -> Result<String, PowerError> {
    Err(PowerError::malformed("state.ini", text))
}

#[cfg(test)]
mod tests;
