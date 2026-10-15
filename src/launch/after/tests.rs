use super::{Priority, after_wait};
use crate::gate::{Evaluation, Overall};
use crate::wait::Outcome;

const PLAIN: Priority = Priority::Normal;
const BACKGROUND: Priority = Priority::Background;
const OR_BACKGROUND: Priority = Priority::OrBackground;

fn unsupported() -> Outcome<Option<crate::slot::Release>> {
    Outcome::Unsupported(Evaluation {
        verdicts: Vec::new(),
        overall: Overall::Unsupported,
    })
}

#[test]
fn released_starts_at_the_asked_priority() {
    let plain = after_wait(Outcome::Released(None), PLAIN);
    assert!(plain.is_ok_and(|s| !s.background && s.release.is_none()));
    let low = after_wait(Outcome::Released(None), BACKGROUND);
    assert!(low.is_ok_and(|s| s.background));
}

#[test]
fn timeout_exits_124() {
    let out = after_wait(Outcome::TimedOut(None), BACKGROUND);
    assert_eq!(out.err(), Some(124));
}

#[test]
fn timeout_with_or_background_starts_in_background() {
    let out = after_wait(Outcome::TimedOut(None), OR_BACKGROUND);
    assert!(out.is_ok_and(|s| s.background && s.release.is_none()));
}

#[test]
fn released_with_or_background_keeps_normal_priority() {
    let out = after_wait(Outcome::Released(None), OR_BACKGROUND);
    assert!(out.is_ok_and(|s| !s.background));
}

#[test]
fn unsupported_exits_69_even_with_or_background() {
    assert_eq!(after_wait(unsupported(), OR_BACKGROUND).err(), Some(69));
}
