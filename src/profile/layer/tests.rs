use std::path::Path;

use crate::profile::{
    Origin, Profile, ProfileError, Switch, TestTable, parse_profiles,
};
use crate::units::parse_name;

/// Profile `name` of the `layer.cgmr` fixture.
fn profile(name: &str) -> Option<Profile> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = dir.join("tests/fixtures/profile/layer.cgmr");
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let file = parse_profiles(&text, &path, Origin::User).ok()?;
    file.get(&parse_name(name).ok()?).cloned()
}

/// The layer of profile `name` with `above` set by flag or env, as argv.
fn layer(name: &str, above: &[&str]) -> Result<Vec<String>, ProfileError> {
    let profile = profile(name).ok_or(ProfileError::Read {
        path: "layer.cgmr".into(),
        reason: format!("no profile {name}"),
    })?;
    let switches = profile.layer(&TestTable, |f| above.contains(&f))?;
    Ok(switches
        .iter()
        .flat_map(Switch::args)
        .map(str::to_owned)
        .collect())
}

#[test]
fn with_nothing_above_every_switch_applies_in_file_order() {
    let argv = [
        "--load", "2", "--mem", "4GB", "--path", "a", "--path", "b", "--quiet",
    ];
    assert_eq!(layer("build", &[]), Ok(argv.map(str::to_owned).to_vec()));
}

#[test]
fn a_switch_set_above_overrides_the_profile_without_error() {
    let argv = ["--mem", "4GB", "--path", "a", "--path", "b", "--quiet"];
    let expected = argv.map(str::to_owned).to_vec();
    assert_eq!(layer("build", &["--load"]), Ok(expected));
}

#[test]
fn a_repeatable_switch_set_above_replaces_all_its_values() {
    let argv = ["--load", "2", "--mem", "4GB", "--quiet"];
    let expected = argv.map(str::to_owned).to_vec();
    assert_eq!(layer("build", &["--path"]), Ok(expected));
}

#[test]
fn a_single_value_switch_twice_in_one_profile_is_an_error() {
    let e = layer("twice", &[]).err();
    assert!(
        matches!(
            &e,
            Some(ProfileError::RepeatedSwitch { at, first: 9, flag })
                if at.line() == 11 && flag == "--load"
        ),
        "{e:?}"
    );
    let above = layer("twice", &["--load"]).err();
    assert!(matches!(above, Some(ProfileError::RepeatedSwitch { .. })));
}

#[test]
fn an_unknown_switch_in_the_profile_is_an_error() {
    let e = layer("typo", &[]).err();
    assert!(
        matches!(&e, Some(ProfileError::UnknownSwitch { at, .. }) if at.line() == 15),
        "{e:?}"
    );
}
