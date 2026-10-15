use std::io;
use std::path::{Path, PathBuf};

use super::{PROJECT_FILE, Places, ReadText, discover};
use crate::profile::{Catalog, Origin, ProfileError};

/// The fixture tree standing for the filesystem.
fn tree() -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    dir.join("tests/fixtures/profile/tree")
}

/// Reads fixture files only: anything outside the tree is not found, so a
/// walk up never reaches the real machine.
fn fixtures(path: &Path) -> io::Result<String> {
    if path.starts_with(tree()) {
        std::fs::read_to_string(path)
    } else {
        Err(io::ErrorKind::NotFound.into())
    }
}

/// Places with the working directory deep inside the fixture repo.
fn places() -> Places {
    Places::new(tree().join("repo/src/deep"))
}

/// Each file in force: its origin and its path relative to the tree.
type Found = Vec<(Origin, PathBuf)>;

/// The origins and paths (relative to the tree) of the files in force.
fn found(places: &Places) -> Result<Found, ProfileError> {
    let catalog = discover(places, &fixtures)?;
    let rel = |p: &Path| p.strip_prefix(tree()).unwrap_or(p).to_owned();
    Ok(Catalog::files(&catalog)
        .iter()
        .map(|f| (f.origin(), rel(f.path())))
        .collect())
}

#[test]
fn the_nearest_project_file_is_found_walking_up() {
    let files = found(&places());
    let project = (Origin::Project, PathBuf::from("repo").join(PROJECT_FILE));
    assert_eq!(files, Ok(vec![project]));
}

#[test]
fn the_user_file_follows_the_project_file() {
    let places = places().config_home(Some(tree().join("xdg")));
    let origins: Result<Vec<Origin>, _> =
        found(&places).map(|f| f.into_iter().map(|(o, _)| o).collect());
    assert_eq!(origins, Ok(vec![Origin::Project, Origin::User]));
}

#[test]
fn the_user_file_is_xdg_else_dot_config() {
    let xdg = Places::new("/").config_home(Some("/x".into()));
    assert_eq!(xdg.user_file(), Some(PathBuf::from("/x/cgmr/profiles")));
    let home = Places::new("/").home(Some("/me".into()));
    let dot = PathBuf::from("/me/.config/cgmr/profiles");
    assert_eq!(home.clone().user_file(), Some(dot.clone()));
    let relative = home.config_home(Some("rel".into()));
    assert_eq!(relative.user_file(), Some(dot));
    assert_eq!(Places::new("/").user_file(), None);
}

#[test]
fn outside_any_project_only_the_user_file_is_read() {
    let places =
        Places::new(tree().join("nowhere")).home(Some(tree().join("me")));
    let files = found(&places);
    let user = PathBuf::from("me/.config/cgmr/profiles");
    assert_eq!(files, Ok(vec![(Origin::User, user)]));
}

#[test]
fn an_explicit_file_is_read_alone() {
    let explicit = tree().join("me/.config/cgmr/profiles");
    let places = places()
        .explicit(Some(explicit))
        .config_home(Some(tree().join("xdg")));
    let files = found(&places);
    let only = (Origin::Explicit, PathBuf::from("me/.config/cgmr/profiles"));
    assert_eq!(files, Ok(vec![only]));
}

#[test]
fn files_not_found_are_skipped() {
    let missing = places().explicit(Some(tree().join("missing")));
    assert_eq!(found(&missing), Ok(Vec::new()));
    let nothing = Places::new(tree().join("nowhere"));
    assert_eq!(found(&nothing), Ok(Vec::new()));
}

/// A reader that is refused every file.
fn denied(path: &Path) -> io::Result<String> {
    let _ = path;
    Err(io::ErrorKind::PermissionDenied.into())
}

#[test]
fn a_file_that_cannot_be_read_is_an_error() {
    let e = discover(&Places::new("/p"), &denied).err();
    let path = PathBuf::from("/p").join(PROJECT_FILE);
    assert!(
        matches!(&e, Some(ProfileError::Read { path: p, .. }) if *p == path),
        "{e:?}"
    );
}

#[test]
fn a_parse_error_in_a_found_file_is_reported() {
    let bad = |_: &Path| Ok("load 2".to_owned());
    let e = discover(&Places::new("/p"), &bad).err();
    assert_eq!(e.and_then(|e| e.at().map(|at| at.line())), Some(1));
}

#[test]
fn a_closure_reads_text() {
    let reader = |_: &Path| Ok("x".to_owned());
    assert_eq!(reader.read_text(Path::new("a")).ok().as_deref(), Some("x"));
}
