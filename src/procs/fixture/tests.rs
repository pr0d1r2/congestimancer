use super::{table, user};
use crate::procs::Proc;

#[test]
fn reads_every_row_and_skips_comments() {
    let procs = table("table");
    assert_eq!(procs.len(), 12);
    assert_eq!(
        procs.first(),
        Some(&Proc::new("launchd", Some(0)).with_lineage(1, 0))
    );
}

#[test]
fn dash_uid_is_none() {
    let procs = table("table");
    let ghost = procs.iter().find(|p| p.name == "ghost");
    assert_eq!(ghost.map(|p| p.uid), Some(None));
}

#[test]
fn missing_fixture_is_empty() {
    assert!(table("no-such-case").is_empty());
}

#[test]
fn knows_its_users() {
    assert_eq!(user(30001).as_deref(), Some("nixbld1"));
    assert_eq!(user(7), None);
}
