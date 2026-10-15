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
    let mut in_state = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            in_state = line == "[State]";
        } else if let Some(profile) = line.strip_prefix("Profile=")
            && in_state
            && !profile.trim().is_empty()
        {
            return Ok(profile.trim().to_owned());
        }
    }
    Err(PowerError::malformed("state.ini", text))
}

#[cfg(test)]
mod tests;
