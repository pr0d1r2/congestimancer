use std::collections::HashSet;

use super::{Proc, count_by_name, lineage};
use crate::procs::Glob;
use crate::procs::fixture::table;
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
    assert_eq!(found, HashSet::from([200, 100, 1]));
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

#[test]
fn a_process_without_lineage_counts() {
    let procs = [Proc::new("cargo", Some(501))];
    assert_eq!(count_by_name(&procs, &Glob::new("car*"), 1), Count(1));
}
