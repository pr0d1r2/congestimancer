use std::collections::HashSet;

use super::{Proc, count_by_name, count_by_user, lineage};
use crate::procs::Glob;
use crate::procs::fixture::{table, user};
use crate::units::Count;

/// cgmr's pid in the `table` fixture.
const ME: u32 = 200;

/// How many processes of the `table` fixture `pattern` names.
fn by_name(pattern: &str) -> Count {
    count_by_name(&table("table"), &Glob::new(pattern), ME)
}

#[test]
fn lineage_is_me_and_my_ancestors() {
    let found = lineage(&table("table"), ME);
    // launchd's parent, pid 0, is an ancestor too, though not listed.
    assert_eq!(found, HashSet::from([200, 100, 1, 0]));
}

#[test]
fn lineage_survives_a_parent_loop() {
    let procs = [
        Proc::new("a", None).with_lineage(5, 6),
        Proc::new("b", None).with_lineage(6, 5),
    ];
    assert_eq!(lineage(&procs, 5), HashSet::from([5, 6]));
}

#[test]
fn lineage_of_an_unlisted_me_is_me() {
    assert_eq!(lineage(&[], 9), HashSet::from([9]));
}

#[test]
fn counts_processes_by_name() {
    assert_eq!(by_name("rustc"), Count(3));
    assert_eq!(by_name("bash"), Count(2));
    assert_eq!(by_name("nothing"), Count(0));
}

#[test]
fn leaves_out_me_and_my_ancestors() {
    // 12 processes, less cgmr (200), zsh (100) and launchd (1).
    assert_eq!(by_name("*"), Count(9));
    // The other cgmr (600) counts; this one does not.
    assert_eq!(by_name("cgmr"), Count(1));
    assert_eq!(by_name("zsh"), Count(0));
}

/// How many processes of the `table` fixture users matching `pattern` own.
fn by_user(pattern: &str) -> Count {
    count_by_user(&table("table"), &Glob::new(pattern), ME, user)
}

#[test]
fn counts_processes_by_user() {
    assert_eq!(by_user("nixbld*"), Count(3));
    assert_eq!(by_user("nixbld1"), Count(1));
    assert_eq!(by_user("nobody"), Count(0));
}

#[test]
fn user_count_leaves_out_me_and_my_ancestors() {
    // alice owns zsh (100) and cgmr (200), both left out.
    assert_eq!(by_user("alice"), Count(4));
    // root owns launchd (1), left out.
    assert_eq!(by_user("root"), Count(1));
}

#[test]
fn no_uid_or_unnamed_uid_never_matches() {
    let procs = [Proc::new("a", None), Proc::new("b", Some(7))];
    let n = count_by_user(&procs, &Glob::new("*"), 1, user);
    assert_eq!(n, Count(0));
}

#[test]
fn names_each_uid_once() {
    let mut asked = Vec::new();
    let names = |uid| {
        asked.push(uid);
        user(uid)
    };
    let _ = count_by_user(&table("table"), &Glob::new("*"), ME, names);
    asked.sort_unstable();
    assert_eq!(asked, [0, 501, 30001, 30002]);
}

#[test]
fn a_process_without_lineage_counts() {
    let procs = [Proc::new("cargo", Some(501))];
    assert_eq!(count_by_name(&procs, &Glob::new("car*"), 1), Count(1));
}
