//! [`Profile::layer`]: a profile's switches as one layer of the config,
//! below the command line and the environment (src/profile:V1, V6).

use std::collections::HashMap;

use super::{Arity, Profile, ProfileError, Switch, SwitchTable};

impl Profile {
    /// The switches this profile adds to the config.
    ///
    /// Precedence is flag > env > profile > builtin default
    /// (src/profile:V1): a switch the command line or the environment
    /// already sets (`set_above` says which, by flag as written) is left
    /// out, never a repeat error (src/profile:V6). For a repeatable switch
    /// the layer above replaces every profile value of it. Every switch is
    /// first checked by the CLI's parser (src/profile:V5).
    ///
    /// # Errors
    ///
    /// [`ProfileError::UnknownSwitch`] or [`ProfileError::BadValue`] from
    /// the check, or [`ProfileError::RepeatedSwitch`] for a single-value
    /// switch set twice in this profile (src/profile:V6).
    pub fn layer(
        &self,
        table: &impl SwitchTable,
        set_above: impl Fn(&str) -> bool,
    ) -> Result<Vec<Switch>, ProfileError> {
        let mut first: HashMap<&str, usize> = HashMap::new();
        let mut kept = Vec::new();
        for switch in self.switches() {
            let (flag, at) = (switch.flag(), switch.at());
            if switch.check(table)? == Arity::Single
                && let Some(first) = first.insert(flag, at.line())
            {
                let (at, flag) = (at.clone(), flag.to_owned());
                return Err(ProfileError::RepeatedSwitch { at, flag, first });
            }
            if !set_above(flag) {
                kept.push(switch.clone());
            }
        }
        Ok(kept)
    }
}

#[cfg(test)]
mod tests;
