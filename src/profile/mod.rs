//! `src/profile`: named switch sets from project & user files, lookup &
//! trust rule.
//!
//! Composes only (src:C): each part lives in its own submodule, and the
//! public surface is re-exported here and again from the crate root
//! (src:V3).

mod catalog;
mod discover;
mod error;
mod file;
mod layer;
mod switch;
mod trust;

pub use catalog::Catalog;
pub use discover::{PROJECT_FILE, Places, ReadText, discover};
pub use error::{At, ProfileError};
pub use file::{Origin, Profile, ProfileFile, parse_profiles};
pub use switch::{Arity, Switch, SwitchFault, SwitchTable};

#[cfg(test)]
pub(crate) use switch::TestTable;
