use std::path::Path;

use super::Catalog;
use crate::profile::{Origin, ProfileError, ProfileFile, parse_profiles};
use crate::units::{Name, parse_name};

/// A name from a literal known to be one.
fn name(s: &str) -> Name {
    parse_name(s).unwrap_or_else(|_| name("bad-literal"))
}

/// A file at `path` from `text`, or an empty one if the text is bad.
fn file(path: &str, origin: Origin, text: &str) -> ProfileFile {
    parse_profiles(text, Path::new(path), origin)
        .unwrap_or_else(|_| file(path, origin, ""))
}

/// A project file with `build` and `ci`, then a user file with `build`
/// and `night`.
fn catalog() -> Catalog {
    Catalog::new(vec![
        file(".cgmr", Origin::Project, "[build]\n--load 2\n[ci]\n"),
        file("profiles", Origin::User, "[night]\n[build]\n--load 8\n"),
    ])
}

/// Where the profile `n` is found, and its first switch's value.
fn found(n: &str) -> Result<(String, Option<String>), ProfileError> {
    let catalog = catalog();
    let (file, profile) = catalog.lookup(&name(n))?;
    let first = profile.switches().first().and_then(|s| s.value());
    Ok((file.path().display().to_string(), first.map(str::to_owned)))
}

#[test]
fn the_project_file_shadows_the_user_file() {
    assert_eq!(found("build"), Ok((".cgmr".into(), Some("2".into()))));
}

#[test]
fn a_name_only_the_user_file_has_is_found_there() {
    assert_eq!(found("night"), Ok(("profiles".into(), None)));
}

#[test]
fn an_unknown_name_lists_every_name_once_sorted() {
    let available = ["build", "ci", "night"].map(name).to_vec();
    assert_eq!(
        found("bild"),
        Err(ProfileError::UnknownProfile {
            name: name("bild"),
            available,
        })
    );
}

#[test]
fn an_empty_catalog_has_no_names() {
    let empty = Catalog::default();
    assert!(empty.names().is_empty());
    assert!(empty.files().is_empty());
    assert_eq!(catalog().files().len(), 2);
}
