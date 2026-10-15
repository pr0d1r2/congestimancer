//! The verbs about cgmr itself: `profiles`, `completions`, `man`, and
//! the hidden `debug capture` (src/cli:I).

use std::path::Path;

use clap_complete::Shell;

use super::args::{Given, command};
use super::exit::{FAIL, Failure};
use super::profiles::{catalog, places};
use super::world::{World, read_file, wrote};
use crate::profile::render_profiles;
use crate::sys::capture;

/// `profiles`: the profile files in force, their names and switches.
///
/// # Errors
///
/// 64 for a profile file that does not parse.
pub(crate) fn profiles(
    world: &mut World<'_>,
    given: &Given,
) -> Result<u8, Failure> {
    let places = places(given, &world.env, &world.cwd)?;
    let catalog = catalog(&places, &read_file)?;
    wrote(world.out.write_all(render_profiles(&catalog).as_bytes()))?;
    Ok(0)
}

/// `completions SHELL`: the completion script, from the same table as
/// argv (src/cli:V7).
///
/// # Errors
///
/// A failed write.
pub(crate) fn completions(
    world: &mut World<'_>,
    shell: Shell,
) -> Result<u8, Failure> {
    let mut cmd = command();
    clap_complete::generate(shell, &mut cmd, "cgmr", &mut world.out);
    wrote(world.out.flush())?;
    Ok(0)
}

/// `man`: the man page in roff, from the same table as argv.
///
/// # Errors
///
/// A failed write.
pub(crate) fn man(world: &mut World<'_>) -> Result<u8, Failure> {
    wrote(clap_mangen::Man::new(command()).render(&mut world.out))?;
    Ok(0)
}

/// `debug capture DIR`: this machine's raw sources as a fixture tree
/// (src/sys:V7); prints each file written.
///
/// # Errors
///
/// 1 when a source cannot be read or DIR cannot be written.
pub(crate) fn capture_into(
    world: &mut World<'_>,
    dir: &Path,
) -> Result<u8, Failure> {
    let written = capture(&world.root, dir)
        .map_err(|e| Failure::new(FAIL, format!("debug capture: {e}")))?;
    for path in written {
        wrote(writeln!(world.out, "{}", path.display()))?;
    }
    Ok(0)
}

#[cfg(test)]
mod tests;
