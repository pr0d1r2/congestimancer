use std::path::Path;

use super::{NONE, render_profiles};
use crate::profile::{Catalog, Origin, ProfileFile, parse_profiles};

/// A file at `path` from `text`, or an empty one if the text is bad.
fn file(path: &str, origin: Origin, text: &str) -> ProfileFile {
    parse_profiles(text, Path::new(path), origin)
        .unwrap_or_else(|_| file(path, origin, ""))
}

/// The project file.
const PROJECT: &str = "[build]\n--load 2\n[ci]\n";

/// The user file.
const USER: &str = "[build]\n--load=8\n[night]\n--when 'test -f /n'\n\
                    --path \"\"\n";

#[test]
fn every_file_lists_its_profiles_and_switches() {
    let catalog = Catalog::new(vec![
        file("/r/.cgmr", Origin::Project, PROJECT),
        file("/u/profiles", Origin::User, USER),
    ]);
    let expected = "/r/.cgmr (project)\n\
                    \x20 [build] --load 2\n\
                    \x20 [ci]\n\
                    /u/profiles (user)\n\
                    \x20 [build] --load 8 (shadowed by /r/.cgmr)\n\
                    \x20 [night] --when \"test -f /n\" --path \"\"\n";
    assert_eq!(render_profiles(&catalog), expected);
}

#[test]
fn an_explicit_file_says_so() {
    let catalog =
        Catalog::new(vec![file("p", Origin::Explicit, "[a]\n--quiet\n")]);
    assert_eq!(render_profiles(&catalog), "p (explicit)\n  [a] --quiet\n");
}

#[test]
fn no_files_say_there_are_no_profiles() {
    assert_eq!(render_profiles(&Catalog::default()), NONE);
}
