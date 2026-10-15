use std::io;

use super::{
    IoPriority, io_priority, ioprio_of, policy_of, priority_of_ioprio,
    priority_of_policy, set_io_priority,
};

#[test]
fn macos_policies_map_both_ways() {
    assert_eq!(policy_of(IoPriority::Default), 0);
    assert_eq!(policy_of(IoPriority::Background), 3);
    assert_eq!(priority_of_policy(3), IoPriority::Background);
    assert_eq!(priority_of_policy(0), IoPriority::Default);
    assert_eq!(priority_of_policy(1), IoPriority::Default);
}

#[test]
fn linux_ioprio_values_carry_the_class_in_the_top_bits() {
    assert_eq!(ioprio_of(IoPriority::Default), (2 << 13) | 4);
    assert_eq!(ioprio_of(IoPriority::Background), 3 << 13);
    assert_eq!(priority_of_ioprio(3 << 13), IoPriority::Background);
    assert_eq!(priority_of_ioprio((2 << 13) | 4), IoPriority::Default);
    assert_eq!(priority_of_ioprio(0), IoPriority::Default);
}

#[test]
fn background_is_set_and_read_back() -> io::Result<()> {
    set_io_priority(IoPriority::Background)?;
    let got = io_priority();
    set_io_priority(IoPriority::Default)?;
    assert_eq!(got?, IoPriority::Background);
    assert_eq!(io_priority()?, IoPriority::Default);
    Ok(())
}
